# V5.0 — Historique des Problèmes et Corrections

> Trace complète de chaque problème rencontré, sa cause racine, sa correction,
> et la vérification effectuée. Sert de mémoire pour les versions futures.

---

## Chronologie

| # | Problème | Sévérité | Corrigé | Vérifié |
|---|---|---|---|---|
| 1 | Accuracy V4 = biais topologique | Conception | V5 anti-biais | Oui |
| 2 | Dijkstra OOM (crash 32 Go) | Critique | `log(C_max/C)` | Oui |
| 3 | invert_mapping non propagé | Critique | `effective_mapping` | Oui |
| 4 | Rollback accidentel VS Code | Critique | 3 fixes re-appliqués | Oui |

---

## Problème 1 : Le biais topologique fondamental

### Symptôme
V4 atteignait 100% d'accuracy avec la géométrie Legacy. Mais en V5, la baseline topology-only (plasticité OFF) atteignait aussi 97% en géométrie symétrique.

### Cause racine
Le kernel exponentiel $w_{ij} = e^{-\lambda d_{ij}}$ avec $\lambda = 0.3$ crée une énorme sensibilité aux différences de distance. Dans la géométrie Legacy :

$$\frac{w(25)}{w(75)} = e^{15} \approx 3.3 \times 10^6$$

Même en géométrie « symétrique » (distances ~29-30), la variance de placement des 50 nœuds par groupe crée des déviations de centroïde de 1-2 unités. Pour $\Delta d = 0.8$ :

$$e^{0.3 \times 0.8} \approx 1.27 \quad \text{(27\% d'avantage)}$$

### Correction
Conception du test anti-biais : **garder la géométrie identique mais inverser le mapping** (`invert_mapping = true`). Si la topologie donne 97% en normal, elle doit donner ~3% en inversé. L'apprentissage doit surmonter ce handicap.

### Vérification
$A_{\text{topo,normal}} + A_{\text{topo,inversé}} = 97.7\% + 2.3\% = 100\%$ ✓
(complémentarité parfaite : chaque essai bascule)

---

## Problème 2 : Dijkstra et les coûts négatifs (crash mémoire)

### Symptôme
Les diagnostics post-simulation (RouteScore) provoquaient une explosion mémoire (>19 Go, crash système avec 32 Go de RAM).

### Cause racine
L'implémentation initiale utilisait :

$$\text{cost}(i \to j) = -\ln(C_{ij})$$

Quand $C_{ij} > 1.0$ (ce qui est courant, les conductances vont jusqu'à 5.0) :

$$-\ln(C_{ij}) < 0$$

Dijkstra suppose des **coûts positifs**. Avec des coûts négatifs, le `BinaryHeap` ne converge pas : on peut toujours trouver un chemin de coût inférieur en ajoutant des arêtes à coût négatif. Le heap grossit indéfiniment → OOM.

### Correction
Remplacer par :

$$\text{cost}(i \to j) = \ln\left(\frac{C_{\max}}{C_{ij}}\right), \quad C_{\max} = 5.0$$

Propriétés :
- $C_{ij} = C_{\max} \Rightarrow \text{cost} = 0$
- $C_{ij} = C_{\min} = 0.1 \Rightarrow \text{cost} = \ln(50) \approx 3.91$
- Tout coût $\geq 0$ ✓

### Fichier modifié
`stretch-core/src/diagnostics.rs`, fonction `trace_best_path`, ligne du calcul de `edge_cost`.

### Vérification
Diagnostics s'exécutent en <1s, usage mémoire stable < 2 Go ✓

---

## Problème 3 : `invert_mapping` non propagé aux essais

### Symptôme
Configuration avec `invert_mapping = true` : le log affichait `mapping=[1, 0]` mais l'accuracy topology-only était 84.8% au lieu de ~3%.

### Cause racine
Dans `setup_v5_training()`, le code faisait :

```rust
self.target_mapping = if config.v5_task.invert_mapping {
    placement.target_mapping.iter().copied().rev().collect()
} else {
    placement.target_mapping.clone()
};

// ... plus tard :
let mut trials = task::generate_trials(
    config.input.num_classes,
    &placement.target_mapping, // ← BUG : utilise le mapping original !
    ...
);
```

`self.target_mapping` était correctement inversé à [1,0] (et affiché dans le log), mais `generate_trials()` recevait `placement.target_mapping` (toujours [0,1]). Les essais (`Trial.target_class`) n'étaient donc _jamais_ inversés.

L'accuracy de 84.8% au lieu de 97% s'explique par le bruit du non-déterminisme rayon, pas par une inversion réelle.

### Correction

```rust
let effective_mapping: Vec<usize> = if config.v5_task.invert_mapping {
    placement.target_mapping.iter().copied().rev().collect()
} else {
    placement.target_mapping.clone()
};
self.target_mapping = effective_mapping.clone();

// ...
let mut trials = task::generate_trials(
    config.input.num_classes,
    &effective_mapping, // ← FIXÉ : utilise le mapping effectif
    ...
);
```

### Fichier modifié
`stretch-core/src/simulation.rs`, fonction `setup_v5_training`.

### Vérification
Avec le fix : topology-only inversé = 2.3%, full learning inversé = 86.3% ✓

---

## Problème 4 : Rollback accidentel VS Code

### Symptôme
Exécution du test `config_v5_baseline_topo.toml` → crash système par OOM (identique au problème 2).

### Cause racine
VS Code a annulé (undo) des modifications dans 3 fichiers, restaurant le code pré-correction :
1. `diagnostics.rs` : retour à `-log(C)` (cause du crash)
2. `config.rs` : champ `invert_mapping` supprimé de `V5TaskConfig`
3. `simulation.rs` : retour à `placement.target_mapping` au lieu de `effective_mapping`

Les 2 fichiers config inversés (`config_v5_sym_inverted_*.toml`) ont aussi été supprimés.

### Correction
Re-application des 3 fixes dans le code + re-création des 2 fichiers config + ajout des sections `[v5_diagnostics]` manquantes.

### Vérification
Build propre (0 warnings, 0 errors) ✓
Tests re-exécutés avec résultats identiques ✓

---

## Leçons Retenues

1. **Dijkstra + coûts négatifs = bombe OOM.** Toujours vérifier que les coûts transformés sont ≥ 0 avant d'utiliser un algorithme de plus court chemin.

2. **Un flag enregistré n'est pas un flag appliqué.** Le fait que `self.target_mapping` soit correct ne signifie pas qu'il est utilisé partout. Toujours tracer la chaîne complète : stockage → utilisation.

3. **Les tests complémentaires détectent les bugs.** Si $A_{\text{normal}} + A_{\text{inversé}} \neq 100\%$ en topology-only, c'est qu'il y a un bug. Ce test de cohérence a immédiatement révélé le problème 3.

4. **VS Code peut annuler silencieusement.** Considérer un commit git après chaque étape critique.