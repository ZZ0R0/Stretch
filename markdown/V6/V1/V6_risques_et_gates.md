# V6 — Risques et gates

## Risques

### R1 : Seuil de sparsité CPU retardé d'un tick
**Probabilité** : Certaine (par design)
**Impact** : Faible — le seuil du tick t-1 est une bonne approximation pour le tick t
car les distributions d'activation sont relativement stables tick-to-tick.
**Mitigation** : Si insuffisant, passer au histogramme GPU (V6.1).

### R2 : Le bonus de nouveauté favorise le bruit
**Probabilité** : Moyenne
**Impact** : Moyen — des neurones activés par le bruit thermique pourraient obtenir
un bonus et déplacer des neurones légitimes.
**Mitigation** : Le seuil d'activation effectif filtre le bruit (threshold > 0.2).
Les neurones avec activation < activation_min sont ignorés dans la compétition.

### R3 : La modulation dopaminergique crée des instabilités
**Probabilité** : Faible
**Impact** : Moyen — si le reverb_max est trop élevé, le réseau pourrait exploser
en phase de recherche.
**Mitigation** : Borner reverb_max à 0.30, clamp total à 5.0 dans le shader.

### R4 : Performance GPU dégradée par le pass de sparsité
**Probabilité** : Faible
**Impact** : Faible — un dispatch supplémentaire de 50k threads est négligeable
par rapport aux 577k threads de la plasticité.
**Mitigation** : Mesurer le temps par tick avant/après.

## Gates

### Gate 0 : Build propre (avant merge)
- [ ] `cargo build --release` : 0 erreurs
- [ ] `cargo clippy` : 0 warnings
- [ ] `cargo test` : 10/10 pass

### Gate 1 : Signal traverse (CA-1)
- [ ] Max readout score > 0.1 dans 80% des trials en mode V6

### Gate 2 : Sparsité respectée (CA-2)
- [ ] Moyenne active_nodes / num_nodes ≤ 0.0525 (5% + 5% marge)

### Gate 3 : Apprentissage (CA-3)
- [ ] Accuracy > 55% sur 5 seeds

### Gate 4 : Non-régression (CA-4)
- [ ] 10/10 tests V5.2 passent sans modification
