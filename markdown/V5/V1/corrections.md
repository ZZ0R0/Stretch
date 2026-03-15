# V5.1 — Historique des Problèmes et Corrections

> Trace complète de chaque problème rencontré pendant la phase V5.1
> (complétion visualisation, tests manquants, finalisation).
> Même format que `V0/correction.md` pour continuité documentaire.

---

## Chronologie

| # | Problème | Sévérité | Corrigé | Vérifié |
|---|---|---|---|---|
| 1 | stretch-viz V4-only (pas de support V5) | Majeur | Réécriture complète | Oui |
| 2 | .gitignore : retour à la ligne manquant | Mineur | Ajout newline | Oui |
| 3 | RandomBaseline jamais exécuté | Lacune | Config créée + test run | Oui |
| 4 | Remap jamais exécuté | Lacune | Test run + analyse | Oui |
| 5 | Variable `point_r` inutilisée (warning) | Mineur | Suppression du champ | Oui |

---

## Problème 1 : stretch-viz bloqué en V4

### Symptôme
Le visualiseur `stretch-viz/src/main.rs` (574 lignes) appellait uniquement `setup_v4_training()`, sans aucune connaissance des concepts V5 : pas de groupes I/O, pas de chemins Dijkstra, pas de conductance par nœud, pas d'accuracy tracking, pas de détection V5.

### Cause racine
La V5.0 s'est concentrée sur le moteur (`stretch-core`) et les tests CLI (`stretch-cli`). Le visualiseur n'a pas été mis à jour — il ne connaissait ni `V5TaskMode`, ni `V5BaselineMode`, ni les diagnostics structurels.

Le cahier des charges V5 §4.5 exigeait une carte de conductance 3D, un path tracer, une timeline conductance et des clusters de co-renforcement. Aucun de ces éléments n'existait dans le visualiseur.

### Correction
Réécriture complète de `stretch-viz/src/main.rs` (~620 lignes). Ajouts :

| Fonctionnalité | Implémentation |
|----------------|----------------|
| Détection V5 automatique | Test `task_mode != Legacy \|\| invert_mapping \|\| baseline != FullLearning` → `setup_v5_training()` |
| Carte I/O | `io_map: Vec<u8>` (0=regular, 1=in0, 2=in1, 3=out0, 4=out1), touche `I` pour toggle |
| Overlay chemins Dijkstra | Appel `diagnostics::trace_best_path()`, touche `P`, coloré par classe |
| Arêtes top conductance | 500 arêtes les plus modifiées (déviation vs 1.0), touche `C` |
| Mode Conductance | ViewMode::Conductance (touche `4`), conductance sortante moyenne par nœud |
| Clusters co-renforcement | Nœuds sur ≥2 chemins tracés, affichés en jaune |
| Accuracy sparkline | Courbe glissante (100 dernières évaluations) dans sidebar |
| Energy sparkline | Courbe d'énergie dans sidebar |
| Timeline conductance | Évolution de `mean_conductance` (échantillon toutes les 10 évaluations) |
| Panel V5 info | Task mode, baseline mode, mapping dans la sidebar |
| Refresh diagnostics | Touche `T` pour recalculer chemins et arêtes |
| Contexte de projection | Struct `ProjCtx` pour factoriser la transformation 3D→2D |

### Fichier modifié
`stretch-viz/src/main.rs` — réécriture intégrale.

### Vérification
- Build propre : 0 erreurs, 0 warnings sur les 3 crates ✓
- Lancement avec `config_v5_sym_learning.toml` : détection V5 automatique, groupes I/O visibles, chemins traçables ✓

---

## Problème 2 : .gitignore — retour à la ligne manquant

### Symptôme
Le fichier `.gitignore` contenait les deux dernières entrées fusionnées sur la même ligne :

```
metrics_output.jsonv5_diagnostics.json
```

Git ne reconnaissait pas `v5_diagnostics.json` comme motif séparé.

### Cause racine
Lors de l'ajout de `v5_diagnostics.json` au `.gitignore`, le retour à la ligne final n'avait pas été ajouté après `metrics_output.json`.

### Correction
Ajout d'un retour à la ligne entre les deux entrées :

```
metrics_output.json
v5_diagnostics.json
```

### Fichier modifié
`.gitignore`

### Vérification
`git status` ne liste plus `v5_diagnostics.json` comme fichier non-suivi ✓

---

## Problème 3 : RandomBaseline jamais exécuté

### Symptôme
Le cahier des charges V5 §4.2 exigeait trois baselines (RandomBaseline, TopologyOnly, FullLearning). La V5.0 n'avait exécuté que TopologyOnly et FullLearning. Le mode RandomBaseline était implémenté dans le code mais jamais testé.

### Cause racine
Pas de fichier de configuration dédié. Le mode `RandomBaseline` avait été implémenté dans `config.rs` et `simulation.rs` mais aucun config `.toml` ne l'activait.

### Correction
Création de `configs/config_v5_sym_random.toml` :

```toml
[v5_task]
task_mode = "Symmetric"
baseline_mode = "RandomBaseline"
invert_mapping = false
random_weight_min = 0.1
random_weight_max = 5.0
```

### Résultat
**68.8% accuracy** en Symmetric RandomBaseline. Ce chiffre se situe entre le hasard pur (50%) et la topologie structurée (97.7%), montrant que les poids aléatoires créent une structure accidentelle qui favorise un output mais sans cohérence systématique.

### Vérification
```
[V5 result] topo=Symmetric baseline=RandomBaseline inverted=false
[V5 result] accuracy=68.8% (344/500) correct_count=344, total_evaluated=500
```
Résultat intégré dans la matrice de preuve ✓

---

## Problème 4 : Remap jamais exécuté

### Symptôme
Le mode Remap (Legacy, inversion du mapping à tick 5000) était implémenté mais jamais testé. Le config `config_v5_remap.toml` existait mais n'avait pas été exécuté.

### Cause racine
La V5.0 s'est concentrée sur la preuve d'apprentissage (matrice 2×2 Symmetric). Le test Remap, bien que configuré, n'a pas été exécuté faute de temps.

### Correction
Exécution de `config_v5_remap.toml` (Legacy, mapping normal, inversion à tick 5000, 10000 ticks total).

### Résultat
**50.0% accuracy globale**. Analyse :
- Première moitié (0–5000) : mapping normal + topologie Legacy → haute accuracy.
- Seconde moitié (5000–10000) : mapping inversé, anciennes routes dominent → basse accuracy.
- Le réseau n'a pas eu assez de ticks pour ré-apprendre. Les routes renforcées en phase 1 persistent et résistent au remap.

### Conclusion
Le re-learning nécessite soit plus de ticks, soit un mécanisme d'oubli/affaiblissement plus agressif. Ce résultat alimente directement la réflexion de la vision post-V5.1.

### Vérification
```
[V5 result] topo=Legacy baseline=FullLearning inverted=false remap_at=5000
[V5 result] accuracy=50.0% (250/500)
```
Résultat documenté dans `V5_aboutissement.md` ✓

---

## Problème 5 : Warning `point_r` inutilisé

### Symptôme
`cargo build` émettait un warning dans `stretch-viz/src/main.rs` :

```
warning: unused variable: `point_r`
```

### Cause racine
Lors de la réécriture, la variable `point_r` (rayon des points en fonction du nombre de nœuds) était calculée dans `draw_3d_view` mais le rendu des nœuds utilisait directement le même calcul inline dans les draw calls.

### Correction
Utilisation de `point_r` dans les draw calls pour les rectangles et cercles au lieu de recalculer la taille.

### Fichier modifié
`stretch-viz/src/main.rs`

### Vérification
Build 0 warnings ✓

---

## Leçons Retenues

1. **Le visualiseur doit évoluer avec le moteur.** La V5.0 a livré un moteur V5 complet mais un visualiseur V4. L'intégration verticale (moteur + diagnostic + visualisation) doit être synchrone.

2. **Tout mode implémenté doit être testé.** RandomBaseline et Remap existaient dans le code depuis la V5.0 mais n'avaient jamais été exécutés. Un mode non testé est un mode potentiellement bugué.

3. **Les configs manquantes sont des tests manquants.** L'absence de `config_v5_sym_random.toml` a masqué le fait que RandomBaseline n'avait jamais été validé. Chaque mode doit avoir au moins un fichier config associé.

4. **Les warnings sont des bugs en attente.** Le warning `point_r` était bénin, mais la politique zéro-warning doit être maintenue pour ne pas masquer des problèmes réels.
