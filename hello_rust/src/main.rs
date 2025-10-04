// src/main.rs

struct Paciente {
    nome: String,
    idade: u32,
    condicao: String,
}

impl Paciente {
    fn novo(nome: &str, idade: u32, condicao: &str) -> Self {
        Paciente {
            nome: nome.to_string(),
            idade,
            condicao: condicao.to_string(),
        }
    }

    fn atender(&self) {
        println!("Atendendo paciente: {} ({}) - Condição: {}", 
                 self.nome, self.idade, self.condicao);
    }
}

fn main() {
    let paciente1 = Paciente::novo("João", 45, "Dor no peito");
    let paciente2 = Paciente::novo("Maria", 32, "Fratura no braço");

    paciente1.atender();
    paciente2.atender();
}
