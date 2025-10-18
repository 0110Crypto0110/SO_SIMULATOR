use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};
use std::time::Duration;
use tokio::time::sleep;

// Funções utilitárias mantidas, mas o core da reserva de recurso com timeout
// está em `recursos.rs` para permitir a detecção de deadlock.

/// Aguarda um recurso específico (função antiga, não usada diretamente pelo paciente)
#[allow(dead_code)]
pub async fn esperar_recurso(nome: &str, recurso: Arc<Semaphore>) -> OwnedSemaphorePermit {
    println!("🕓 Aguardando recurso: {}", nome);
    let permit = recurso.acquire_owned().await.unwrap();
    println!("✅ {} adquirido com sucesso!", nome);
    permit
}

/// Simula o uso de um recurso por um determinado tempo (com log)
pub async fn usar_recurso(nome: &str, duracao: u64) {
    println!("🔧 Utilizando recurso: {} por {} segundos...", nome, duracao);
    sleep(Duration::from_secs(duracao)).await;
    println!("🏁 Liberação do recurso: {}", nome);
}

/// Função auxiliar para pausar a execução entre etapas (para logs visíveis)
pub async fn pausa(segundos: u64) {
    sleep(Duration::from_secs(segundos)).await;
}

// Estrutura Sincronizacao não é usada, mas pode ser mantida
#[allow(dead_code)]
pub struct Sincronizacao {
    sem: Arc<Semaphore>,
}

#[allow(dead_code)]
impl Sincronizacao {
    pub fn novo(n: usize) -> Self {
        Self { sem: Arc::new(Semaphore::new(n)) }
    }

    pub fn executar<F, Fut>(&self, f: F) -> tokio::task::JoinHandle<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let sem = self.sem.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();
            f().await;
        })
    }
}