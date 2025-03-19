# Rust - IDS

## Auteurs
- **Thomas LAMBERT**
- **Esteban PERRIN**

## Classe
**3SI3**

## Description du Projet
Ce projet est un **Système de Détection d'Intrusions (IDS)** développé en **Rust**. Il permet de surveiller les logs et le trafic réseau pour détecter des activités suspectes telles que :

- **Tentatives de connexion SSH par force brute**
- **Scans de ports effectués avec des outils comme Nmap ou Masscan**
- **Détection d'attaques web via l'analyse des logs Apache/Nginx**

Le système enregistre les alertes, les affiche via une **interface web en Rust** et permet de bloquer automatiquement les adresses IP malveillantes via `iptables`.

## Objectifs
- Lire et analyser les logs en temps réel
- Surveiller le trafic réseau et détecter les scans
- Fournir une interface web en Rust pour visualiser et gérer les alertes
- Offrir des options de blocage automatique des IP suspectes

---

## Installation et Utilisation
(Section à compléter une fois le développement avancé)

```bash
# Cloner le dépôt
git clone https://github.com/Thixo7/Rust.git
cd Rust

# Compiler le backend
cd backend
cargo build --release

# Lancer l'IDS
cargo run
```
