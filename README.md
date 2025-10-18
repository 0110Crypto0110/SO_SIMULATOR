

````markdown
# 🏥 SO_SIMULATOR

Simulador de gerenciamento hospitalar em Rust, com monitoramento de recursos, progresso de pacientes e logs em tempo real.

---

## 🔹 Instalação do Rust no Windows
=======================================================================
Baixe o instalador oficial:
=======================================================================

👉 [https://win.rustup.rs](https://win.rustup.rs)

Ele baixa o `rustup-init.exe`.  

Execute o instalador (pode aceitar a configuração padrão).  
Isso instala `rustc`, `cargo` e `rustup`.

=======================================================================
Depois de instalado, feche e reabra o PowerShell/VS Code e digite:
=======================================================================

```bash
rustc --version
cargo --version
````

Se responder com a versão, tudo está pronto. ✅

---

## 🔹 Verificação final

=======================================================================

Digite:

```bash
cargo new hello_rust
cd hello_rust
cargo run
```

Isso deve criar um projeto Rust, compilar e rodar o clássico Hello, world!
O arquivo `main` está em `hello_rust/src/main.rs`.

---

## 🔹 Como executar

=======================================================================

No terminal do projeto:

```bash
cargo run
```

---

## 📋 Saída esperada

=======================================================================

```text
Atendendo paciente: João (45) - Condição: Dor no peito
Atendendo paciente: Maria (32) - Condição: Fratura no braço
```

---

## 🔹 Extensões principais

=======================================================================

**Rust Analyzer (mais importante ✅)**
Fornece intellisense, autocompletar, navegação no código, diagnósticos e muito mais.

**CodeLLDB**
Necessário se quiser depurar (debug) código Rust dentro do VS Code.

**tokio**

```bash
cargo add tokio --features full
```

---

## 🔹 Anotações

=======================================================================
##indireitei a logica para o funcionamento ficar como ela pediu no docs;

##falta implementar a representacao e talvez as listas dos outros recursos alem de medicos;
##falta completar a mecanica de prioridade;
