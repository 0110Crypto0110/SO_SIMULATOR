use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, OwnedSemaphorePermit};
use tokio::time::{Duration, timeout, Instant};

pub struct Recursos {
    pub medicos: Arc<Semaphore>,
    pub salas_cirurgia: Arc<Semaphore>,
    pub leitos: Arc<Semaphore>,
    pub fila_medicos: Arc<Mutex<Vec<String>>>,
    pub fila_salas: Arc<Mutex<Vec<String>>>,
    pub fila_leitos: Arc<Mutex<Vec<String>>>,
    pub deadlock_medicos: Arc<Mutex<Vec<String>>>,
    pub deadlock_salas: Arc<Mutex<Vec<String>>>,
    pub deadlock_leitos: Arc<Mutex<Vec<String>>>,
}

impl Recursos {
    pub fn novo(qtd_medicos: usize, qtd_salas: usize, qtd_leitos: usize) -> Self {
        Self {
            medicos: Arc::new(Semaphore::new(qtd_medicos)),
            salas_cirurgia: Arc::new(Semaphore::new(qtd_salas)),
            leitos: Arc::new(Semaphore::new(qtd_leitos)),
            fila_medicos: Arc::new(Mutex::new(vec![])),
            fila_salas: Arc::new(Mutex::new(vec![])),
            fila_leitos: Arc::new(Mutex::new(vec![])),
            deadlock_medicos: Arc::new(Mutex::new(vec![])),
            deadlock_salas: Arc::new(Mutex::new(vec![])),
            deadlock_leitos: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn reservar_recurso(
        fila: &Arc<Mutex<Vec<String>>>,
        sem: Arc<Semaphore>,
        deadlock: &Arc<Mutex<Vec<String>>>,
        nome: String,
        timeout_secs: u64,
    ) -> Option<OwnedSemaphorePermit> {
        // Adiciona à fila se ainda não estiver
        {
            let mut f = fila.lock().unwrap();
            if !f.contains(&nome) {
                f.push(nome.clone());
            }
        }

        let start_time = Instant::now();

        loop {
            // Tenta adquirir o recurso com timeout pequeno
            let tentativa = timeout(Duration::from_secs(1), sem.clone().acquire_owned()).await;

            match tentativa {
                Ok(permit) => {
                    // Sucesso → remove da fila e deadlock
                    {
                        let mut f = fila.lock().unwrap();
                        f.retain(|x| x != &nome);
                    }
                    {
                        let mut dl = deadlock.lock().unwrap();
                        dl.retain(|x| x != &nome);
                    }
                    return Some(permit.unwrap());
                }
                Err(_) => {
                    // Timeout da tentativa → verifica se já passou do limite
                    if start_time.elapsed() >= Duration::from_secs(timeout_secs) {
                        let mut dl = deadlock.lock().unwrap();
                        if !dl.contains(&nome) {
                            dl.push(nome.clone());
                        }
                        return None;
                    }
                    // Senão, continua tentando
                }
            }
        }
    }

    pub async fn reservar_medico(&self, nome: String) -> Option<OwnedSemaphorePermit> {
        Self::reservar_recurso(
            &self.fila_medicos,
            self.medicos.clone(),
            &self.deadlock_medicos,
            nome,
            10, // tempo total maior
        ).await
    }

    pub async fn reservar_sala(&self, nome: String) -> Option<OwnedSemaphorePermit> {
        Self::reservar_recurso(
            &self.fila_salas,
            self.salas_cirurgia.clone(),
            &self.deadlock_salas,
            nome,
            10,
        ).await
    }

    pub async fn reservar_leito(&self, nome: String) -> Option<OwnedSemaphorePermit> {
        Self::reservar_recurso(
            &self.fila_leitos,
            self.leitos.clone(),
            &self.deadlock_leitos,
            nome,
            10,
        ).await
    }
}
