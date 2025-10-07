use std::sync::Arc;
use tokio::sync::Semaphore;

/// Estrutura que gerencia todos os recursos do hospital
pub struct Recursos {
    pub medicos: Arc<Semaphore>,
    pub salas_cirurgia: Arc<Semaphore>,
    pub leitos: Arc<Semaphore>,
}

impl Recursos {
    /// Inicializa o conjunto de recursos disponíveis no hospital
    pub fn novo(qtd_medicos: usize, qtd_salas: usize, qtd_leitos: usize) -> Self {
        Recursos {
            medicos: Arc::new(Semaphore::new(qtd_medicos)),
            salas_cirurgia: Arc::new(Semaphore::new(qtd_salas)),
            leitos: Arc::new(Semaphore::new(qtd_leitos)),
        }
    }

    /// Exibe o status atual dos recursos (opcional para monitoramento)
    pub fn status(&self) {
        println!("📊 Recursos disponíveis:");
        println!("  Médicos: {}", self.medicos.available_permits());
        println!("  Salas de Cirurgia: {}", self.salas_cirurgia.available_permits());
        println!("  Leitos: {}", self.leitos.available_permits());
    }
}
