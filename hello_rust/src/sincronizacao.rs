use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};
use std::time::Duration;
use tokio::time::sleep;

/// Aguarda um recurso específico (médico, sala, leito)
/// e retorna o *permit* que deve ser mantido enquanto o recurso está em uso.
pub async fn esperar_recurso(nome: &str, recurso: Arc<Semaphore>) -> OwnedSemaphorePermit {
    println!("🕓 Aguardando recurso: {}", nome);
    let permit = recurso.acquire_owned().await.unwrap();
    println!("✅ {} adquirido com sucesso!", nome);
    permit
}

/// Simula o uso de um recurso por um determinado tempo
pub async fn usar_recurso(nome: &str, duracao: u64) {
    println!("🔧 Utilizando recurso: {} por {} segundos...", nome, duracao);
    sleep(Duration::from_secs(duracao)).await;
    println!("🏁 Liberação do recurso: {}", nome);
}

/// Função auxiliar para pausar a execução entre etapas (para logs visíveis)
pub async fn pausa(segundos: u64) {
    sleep(Duration::from_secs(segundos)).await;
}
