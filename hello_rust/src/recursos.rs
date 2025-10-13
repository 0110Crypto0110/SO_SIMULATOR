use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Recursos {
    pub medicos: Arc<Semaphore>,
    pub salas_cirurgia: Arc<Semaphore>,
    pub leitos: Arc<Semaphore>,
    pub uso_medicos: Arc<Mutex<Vec<String>>>,
    pub fila_medicos: Arc<Mutex<VecDeque<String>>>,
    pub uso_salas: Arc<Mutex<Vec<String>>>,
    pub fila_salas: Arc<Mutex<VecDeque<String>>>,
    pub uso_leitos: Arc<Mutex<Vec<String>>>,
    pub fila_leitos: Arc<Mutex<VecDeque<String>>>,
}

impl Recursos {
    pub fn novo(qtd_medicos: usize, qtd_salas: usize, qtd_leitos: usize) -> Self {
        Recursos {
            medicos: Arc::new(Semaphore::new(qtd_medicos)),
            salas_cirurgia: Arc::new(Semaphore::new(qtd_salas)),
            leitos: Arc::new(Semaphore::new(qtd_leitos)),
            uso_medicos: Arc::new(Mutex::new(vec![])),
            fila_medicos: Arc::new(Mutex::new(VecDeque::new())),
            uso_salas: Arc::new(Mutex::new(vec![])),
            fila_salas: Arc::new(Mutex::new(VecDeque::new())),
            uso_leitos: Arc::new(Mutex::new(vec![])),
            fila_leitos: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn adiciona_uso(&self, tipo: &str, nome: &str) {
        let lista = match tipo {
            "Médico" => &self.uso_medicos,
            "Sala de Cirurgia" => &self.uso_salas,
            "Leito" => &self.uso_leitos,
            _ => return,
        };
        lista.lock().unwrap().push(nome.to_string());
    }

    pub fn libera_recurso(&self, tipo: &str, nome: &str) {
        let lista = match tipo {
            "Médico" => &self.uso_medicos,
            "Sala de Cirurgia" => &self.uso_salas,
            "Leito" => &self.uso_leitos,
            _ => return,
        };
        let mut l = lista.lock().unwrap();
        l.retain(|x| x != nome);
    }

    pub fn adiciona_fila(&self, tipo: &str, nome: &str) {
        let fila = match tipo {
            "Médico" => &self.fila_medicos,
            "Sala de Cirurgia" => &self.fila_salas,
            "Leito" => &self.fila_leitos,
            _ => return,
        };
        fila.lock().unwrap().push_back(nome.to_string());
    }

    pub fn remove_fila(&self, tipo: &str, nome: &str) {
        let fila = match tipo {
            "Médico" => &self.fila_medicos,
            "Sala de Cirurgia" => &self.fila_salas,
            "Leito" => &self.fila_leitos,
            _ => return,
        };
        let mut f = fila.lock().unwrap();
        f.retain(|x| x != nome);
    }
}
