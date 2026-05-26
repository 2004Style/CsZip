//! Barra de progreso para operaciones de compresión/descompresión
//!
//! Proporciona una interfaz para mostrar el progreso de operaciones
//! en la línea de comandos.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::utils::{format_duration, format_size, format_throughput, throughput};

/// Estilo de la barra de progreso
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressStyle {
    /// Barra simple: [=====>    ] 50%
    #[default]
    Bar,
    /// Spinner para operaciones sin tamaño conocido
    Spinner,
    /// Solo porcentaje
    Percentage,
    /// Detallado con velocidad
    Detailed,
    /// Sin salida visual
    Hidden,
}

/// Configuración de la barra de progreso
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Estilo de visualización
    pub style: ProgressStyle,
    /// Ancho de la barra en caracteres
    pub width: usize,
    /// Intervalo mínimo entre actualizaciones
    pub update_interval: Duration,
    /// Mostrar velocidad
    pub show_speed: bool,
    /// Mostrar ETA
    pub show_eta: bool,
    /// Prefijo del mensaje
    pub prefix: String,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            style: ProgressStyle::Bar,
            width: 40,
            update_interval: Duration::from_millis(100),
            show_speed: true,
            show_eta: true,
            prefix: String::new(),
        }
    }
}

impl ProgressConfig {
    /// Crear configuración con estilo específico
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Establecer prefijo
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Establecer ancho
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }
}

/// Barra de progreso
pub struct ProgressBar {
    config: ProgressConfig,
    total: Option<u64>,
    current: u64,
    start_time: Instant,
    last_update: Instant,
    spinner_frame: usize,
    finished: bool,
}

impl ProgressBar {
    /// Crear nueva barra de progreso
    pub fn new(total: Option<u64>) -> Self {
        Self {
            config: ProgressConfig::default(),
            total,
            current: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            spinner_frame: 0,
            finished: false,
        }
    }

    /// Crear con configuración personalizada
    pub fn with_config(total: Option<u64>, config: ProgressConfig) -> Self {
        Self {
            config,
            total,
            current: 0,
            start_time: Instant::now(),
            last_update: Instant::now(),
            spinner_frame: 0,
            finished: false,
        }
    }

    /// Crear barra oculta (sin salida)
    pub fn hidden() -> Self {
        Self::with_config(
            None,
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        )
    }

    /// Establecer progreso absoluto
    pub fn set(&mut self, value: u64) {
        self.current = value;
        self.maybe_render();
    }

    /// Incrementar progreso
    pub fn inc(&mut self, delta: u64) {
        self.current += delta;
        self.maybe_render();
    }

    /// Establecer total
    pub fn set_total(&mut self, total: u64) {
        self.total = Some(total);
    }

    /// Establecer mensaje/prefijo
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.config.prefix = message.into();
    }

    /// Renderizar si ha pasado suficiente tiempo
    fn maybe_render(&mut self) {
        if self.config.style == ProgressStyle::Hidden {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.config.update_interval {
            self.render();
            self.last_update = now;
        }
    }

    /// Renderizar la barra
    fn render(&mut self) {
        if self.config.style == ProgressStyle::Hidden {
            return;
        }

        let output = match self.config.style {
            ProgressStyle::Bar => self.render_bar(),
            ProgressStyle::Spinner => self.render_spinner(),
            ProgressStyle::Percentage => self.render_percentage(),
            ProgressStyle::Detailed => self.render_detailed(),
            ProgressStyle::Hidden => return,
        };

        // Escribir con retorno de carro para sobrescribir
        let _ = write!(io::stderr(), "\r{}", output);
        let _ = io::stderr().flush();
    }

    /// Renderizar barra de progreso
    fn render_bar(&self) -> String {
        let percentage = self.percentage();
        let filled = (percentage / 100.0 * self.config.width as f64) as usize;
        let empty = self.config.width.saturating_sub(filled);

        let bar: String = std::iter::repeat_n('=', filled.saturating_sub(1))
            .chain(if filled > 0 { Some('>') } else { None })
            .chain(std::iter::repeat_n(' ', empty))
            .collect();

        format!(
            "{}[{}] {:5.1}% {}",
            if self.config.prefix.is_empty() {
                String::new()
            } else {
                format!("{} ", self.config.prefix)
            },
            bar,
            percentage,
            self.speed_string()
        )
    }

    /// Renderizar spinner
    fn render_spinner(&mut self) -> String {
        const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let frame = FRAMES[self.spinner_frame % FRAMES.len()];
        self.spinner_frame += 1;

        format!(
            "{}{} {} {}",
            if self.config.prefix.is_empty() {
                String::new()
            } else {
                format!("{} ", self.config.prefix)
            },
            frame,
            format_size(self.current),
            self.speed_string()
        )
    }

    /// Renderizar solo porcentaje
    fn render_percentage(&self) -> String {
        format!(
            "{}{:5.1}%",
            if self.config.prefix.is_empty() {
                String::new()
            } else {
                format!("{} ", self.config.prefix)
            },
            self.percentage()
        )
    }

    /// Renderizar detallado
    fn render_detailed(&self) -> String {
        let elapsed = self.elapsed();
        let eta = self.eta();

        let mut parts = vec![
            format!("{:5.1}%", self.percentage()),
            format!(
                "{}/{}",
                format_size(self.current),
                self.total.map_or("?".to_string(), format_size)
            ),
        ];

        if self.config.show_speed {
            parts.push(self.speed_string());
        }

        parts.push(format!(
            "Elapsed: {}",
            format_duration(elapsed.as_millis() as u64)
        ));

        if self.config.show_eta {
            if let Some(eta) = eta {
                parts.push(format!("ETA: {}", format_duration(eta.as_millis() as u64)));
            }
        }

        format!(
            "{}{}",
            if self.config.prefix.is_empty() {
                String::new()
            } else {
                format!("{} ", self.config.prefix)
            },
            parts.join(" | ")
        )
    }

    /// Calcular porcentaje
    fn percentage(&self) -> f64 {
        match self.total {
            Some(total) if total > 0 => (self.current as f64 / total as f64) * 100.0,
            Some(_) => 100.0,
            None => 0.0,
        }
    }

    /// Calcular velocidad actual
    fn speed(&self) -> f64 {
        let elapsed = self.elapsed().as_millis() as u64;
        throughput(self.current, elapsed)
    }

    /// String de velocidad
    fn speed_string(&self) -> String {
        if self.config.show_speed {
            format_throughput(self.speed())
        } else {
            String::new()
        }
    }

    /// Tiempo transcurrido
    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Tiempo estimado restante
    fn eta(&self) -> Option<Duration> {
        if let Some(total) = self.total {
            if self.current > 0 && self.current < total {
                let elapsed = self.elapsed().as_secs_f64();
                let rate = self.current as f64 / elapsed;
                let remaining = (total - self.current) as f64 / rate;
                return Some(Duration::from_secs_f64(remaining));
            }
        }
        None
    }

    /// Finalizar la barra
    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;

        if self.config.style != ProgressStyle::Hidden {
            self.render();
            eprintln!(); // Nueva línea al final
        }
    }

    /// Finalizar con mensaje
    pub fn finish_with_message(&mut self, message: &str) {
        if self.finished {
            return;
        }
        self.finished = true;

        if self.config.style != ProgressStyle::Hidden {
            let _ = write!(io::stderr(), "\r{}\n", message);
            let _ = io::stderr().flush();
        }
    }

    /// Limpiar línea
    pub fn clear(&self) {
        if self.config.style != ProgressStyle::Hidden {
            let _ = write!(io::stderr(), "\r{}\r", " ".repeat(80));
            let _ = io::stderr().flush();
        }
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if !self.finished && self.config.style != ProgressStyle::Hidden {
            self.finish();
        }
    }
}

/// Crear barra de progreso simple
pub fn progress_bar(total: u64) -> ProgressBar {
    ProgressBar::new(Some(total))
}

/// Crear spinner
pub fn spinner() -> ProgressBar {
    ProgressBar::with_config(
        None,
        ProgressConfig::default().with_style(ProgressStyle::Spinner),
    )
}

/// Macro para crear barra con mensaje
#[macro_export]
macro_rules! progress {
    ($total:expr, $msg:expr) => {{
        let mut pb = $crate::cli::progress::ProgressBar::new(Some($total));
        pb.set_message($msg);
        pb
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_new() {
        let pb = ProgressBar::new(Some(100));
        assert_eq!(pb.current, 0);
        assert_eq!(pb.total, Some(100));
    }

    #[test]
    fn test_progress_bar_set() {
        let mut pb = ProgressBar::with_config(
            Some(100),
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        pb.set(50);
        assert_eq!(pb.current, 50);
    }

    #[test]
    fn test_progress_bar_inc() {
        let mut pb = ProgressBar::with_config(
            Some(100),
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        pb.inc(10);
        pb.inc(20);
        assert_eq!(pb.current, 30);
    }

    #[test]
    fn test_progress_bar_percentage() {
        let mut pb = ProgressBar::with_config(
            Some(100),
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        pb.set(50);
        assert_eq!(pb.percentage(), 50.0);
    }

    #[test]
    fn test_progress_bar_percentage_zero_total() {
        let mut pb = ProgressBar::with_config(
            Some(0),
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        pb.set(0);
        assert_eq!(pb.percentage(), 100.0);
    }

    #[test]
    fn test_progress_bar_no_total() {
        let pb = ProgressBar::with_config(
            None,
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        assert_eq!(pb.percentage(), 0.0);
    }

    #[test]
    fn test_progress_config_builder() {
        let config = ProgressConfig::default()
            .with_style(ProgressStyle::Detailed)
            .with_prefix("Test")
            .with_width(50);

        assert_eq!(config.style, ProgressStyle::Detailed);
        assert_eq!(config.prefix, "Test");
        assert_eq!(config.width, 50);
    }

    #[test]
    fn test_hidden_progress() {
        let mut pb = ProgressBar::hidden();
        pb.set(50);
        pb.inc(25);
        pb.finish();
        // No debe producir salida
    }

    #[test]
    fn test_render_bar() {
        let mut pb = ProgressBar::with_config(
            Some(100),
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        pb.set(50);
        let output = pb.render_bar();
        assert!(output.contains("50.0%"));
    }

    #[test]
    fn test_render_percentage() {
        let mut pb = ProgressBar::with_config(
            Some(100),
            ProgressConfig::default().with_style(ProgressStyle::Hidden),
        );
        pb.set(75);
        let output = pb.render_percentage();
        assert!(output.contains("75.0%"));
    }
}
