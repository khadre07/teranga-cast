# TerangaCast - Alternative Sans Fil au Câble HDMI

**Projet de partage d'écran ordinateur vers télévision**  
Solution logicielle 100% locale (réseau Wi-Fi) pour diffuser l'écran de l'ordinateur sur la TV sans aucun câble.

**Auteur** : Développeur Sénégalais (Rust, Go, React)  
**Objectif** : Offrir une alternative simple, gratuite et performante au câble HDMI physique, adaptée au contexte sénégalais.

---

## Contexte et Positionnement

De nombreuses personnes au Sénégal utilisent encore un câble HDMI pour relier leur ordinateur à la télévision (films, matchs, cours, présentations).  
Bien que la plupart des TVs modernes intègrent des fonctions de mirroring (Miracast, Smart View, AirPlay, Chromecast built-in), celles-ci sont souvent instables, limitées par marque, ou présentent trop de latence.

**TerangaCast** vise à proposer une solution plus universelle, stable et optimisée pour les réseaux locaux sénégalais.

---

## Choix Techniques

- **Langage principal** : **Rust** (pour les meilleures performances, faible latence et faible consommation CPU)
- **Interface sur PC** : Tauri 2 (Rust backend + React frontend) → Application desktop native
- **Récepteur sur TV** : Page web simple (HTML + JavaScript WebRTC)
- **Nombre de connexions** : Commencer avec **1 seul ordinateur** (sender) vers 1 TV
- **Fonctionnement** : 100% en réseau local (LAN) — aucun serveur distant ni internet requis

**Important** : L’ordinateur n’a pas besoin d’avoir un navigateur ouvert. L’utilisateur lancera simplement l’application TerangaCast comme n’importe quel logiciel.

---

## Pourquoi développer ce projet malgré les fonctionnalités intégrées dans les TVs ?

- Meilleure stabilité et latence sur les réseaux Wi-Fi sénégalais
- Universel (moins de problèmes de compatibilité entre marques)
- Pas de pubs ni de limitations
- Confidentialité totale (aucune donnée envoyée à Google, Samsung, Amazon…)
- Possibilité d’adapter l’application au contexte local (français/wolof, mode économie de données, etc.)
- Opportunité réelle : très peu (voire aucun) concurrent sénégalais sur ce créneau

---

## Structure recommandée du projet

- **sender/** → Application principale (PC) avec Tauri + Rust
- **receiver/** → Page web simple pour la TV
- **common/** → Code et types partagés
- **crates/** → Bibliothèques internes (capture, encoder, webrtc, discovery)
- **ui/** → Composants React

---

## Fonctionnalités du MVP (Phase 1)

- Capture d’écran en temps réel
- Encodage vidéo avec accélération hardware (H.264)
- Transmission du flux via WebRTC
- Découverte automatique du PC via mDNS
- Interface simple sur PC (bouton Démarrer / Arrêter, choix de qualité)
- Page web sur TV pour recevoir le flux
- Support Windows en priorité (le plus utilisé au Sénégal)

---

## Avantages pour le marché sénégalais

- Solution gratuite ou très abordable
- Pas besoin d’acheter de dongle Chromecast ou Fire TV Stick
- Fonctionne même avec des connexions Wi-Fi modestes
- Potentiel pour écoles, cybercafés, familles, formations et projections de matchs

---

## Roadmap de développement

**Phase 1 (MVP - 4 à 8 semaines)**  
→ Capture + Encodage + Streaming WebRTC (1 ordinateur)

**Phase 2**  
→ Interface Tauri + React + Amélioration latence + Audio

**Phase 3**  
→ Découverte automatique + Support multi-TV + Version 2 (jusqu’à 3 ordinateurs)

---

**Nom du fichier** : 


