# Vision post-V2 — État du système et direction

Ce document est destiné à être fourni à un LLM pour contextualiser la suite de la roadmap. Il résume de façon factuelle **ce que le système est**, **ce qu'il sait faire**, **ce qu'il ne sait pas faire**, et **ce qu'il devrait devenir**.

---

## 1. Ce que Stretch est aujourd'hui (fin V2)

### 1.1 Nature du système

Stretch est un **substrat de dynamiques spatio-temporelles** simulé en Rust. C'est un graphe spatial 3D de 50 000 nœuds connectés par ~577 000 arêtes pondérées, dans lequel une activité numérique se propage, se dissipe, et se renforce par plasticité Hebbienne.

Ce n'est **pas** un réseau de neurones artificiels classique (pas de couches, pas de backpropagation, pas de fonction de loss). C'est un simulateur de dynamiques continues inspiré des substrats biologiques.

### 1.2 Ce qu'il sait faire

| Capacité | Mécanisme | Qualité |
|---|---|---|
| Activité auto-entretenue | PID par zones | ✅ Stable, indéfinie |
| Oscillations périodiques | Pacemakers sinusoïdaux | ✅ Multiples fréquences |
| Propagation spatiale | Noyaux exponentiels/gaussiens | ✅ Calibrée |
| Mémoire structurelle | Consolidation d'arêtes | ⚠️ Fonctionne mais non sélective |
| Plasticité | Hebbienne corrélative | ⚠️ Pas de causalité temporelle |
| Visualisation | Projection 3D temps réel | ✅ 50k nœuds, ~45 FPS |
| Reproductibilité | ChaCha8 + seed | ✅ Déterministe |

### 1.3 Ce qu'il ne sait pas faire

| Incapacité | Raison | Impact |
|---|---|---|
| Assemblées neuronales | Pas d'inhibition latérale | Pas de représentation d'objets/concepts |
| Compétition entre patterns | Pas de winner-take-all | Pas de sélection / attention |
| Apprentissage de séquences | Pas de STDP | Pas de mémoire procédurale |
| Spécialisation régionale | PID uniforme | Pas de division fonctionnelle |
| Oscillations émergentes | Pas de boucles E/I | Oscillations imposées, pas naturelles |
| Mémoire de travail | Pas de réactivation de patterns | Pas de maintien intentionnel |
| Passage à l'échelle | CPU monothread | Plafonné à ~50k nœuds temps réel |

---

## 2. Les 6 variables d'état par nœud

Chaque nœud possède :

```
activation      [0, 10]     amplitude du signal
fatigue         [0, 10]     réfractorité croissante avec l'activité
inhibition      [0, 10]     auto-inhibition proportionnelle
memory_trace    [0, 10]     trace mnésique à décroissance lente (τ ≈ 138 ticks)
excitability    [0.5, 2.0]  modulation du seuil par la trace
threshold                   seuil effectif θ_eff = θ / ε + f + h
```

Ces 6 variables sont identiques pour tous les nœuds. Il n'y a **aucune différenciation** de type : pas de neurone excitateur vs inhibiteur, pas de neurone rapide vs lent, pas de neurone mémoriel vs relay.

---

## 3. Les mécanismes V2 et leurs limites

### 3.1 PID — Force et faiblesse

**Force** : Le PID résout définitivement le problème d'extinction. C'est un mécanisme robust.

**Faiblesse** : Le PID injecte directement de l'activation ($a_i += u$). Cela court-circuite la dynamique de propagation : le réseau n'a pas besoin de propager pour être actif, il est perfusé par le contrôleur. Si on coupe le PID, tout s'éteint en ~20 ticks.

**Vision** : Le PID devrait évoluer vers un mécanisme **indirect** qui modifie les conditions locales (seuil, gain, excitabilité) plutôt que l'activation elle-même. Le réseau doit apprendre à s'auto-entretenir par ses propres boucles récurrentes, avec le PID comme filet de sécurité, pas comme source d'énergie principale.

### 3.2 Consolidation — Tout ou rien

**Force** : Le mécanisme fonctionne : les arêtes deviennent permanentes.

**Faiblesse** : Avec le PID qui maintient une activité uniforme, toutes les arêtes se consolident. La mémoire n'est pas sélective.

**Vision** : La consolidation doit devenir compétitive. Trois approches possibles :
1. **Normalisation par nœud** : $\sum_j w_{ij} = W_{\text{budget}}$ — les arêtes se font concurrence pour un budget fini.
2. **Seuil adaptatif** : le seuil de consolidation monte avec la conductance moyenne (seules les arêtes exceptionnelles se consolident).
3. **Gating par activité distincte** : ne consolider que pendant des "événements" (activation nettement supérieure au fond PID), pas pendant l'activité de fond.

### 3.3 Pacemakers — Simples mais efficaces

**Force** : Créent des oscillations locales et des battements par interférence.

**Faiblesse** : Ce sont des générateurs de signal externes, pas des oscillateurs émergents. Les vraies oscillations cérébrales naissent de la dynamique E/I, pas de sinusoïdes injectées.

**Vision** : Les pacemakers V2 devraient rester comme "amorces" mais être remplacés progressivement par des oscillations émergentes issues de circuits E→I→E récurrents.

---

## 4. Ce que V3 doit accomplir

### 4.1 Objectif central

Transformer le réseau de "substrat irrigué par un contrôleur" en "substrat dont les dynamiques émergent de l'interaction entre types neuronaux".

### 4.2 Les trois piliers manquants

#### Pilier 1 : Inhibition

Environ 20% des nœuds doivent être inhibiteurs :
- Quand un neurone inhibiteur s'active, il **réduit** l'activation de ses voisins.
- Cela crée du contraste spatial, de la compétition, et des oscillations émergentes.
- Le circuit E→I→E est le mécanisme de base des oscillations gamma (30–80 Hz) dans le cortex.

Implémentation minimale :
```
node.type = Excitatory | Inhibitory
propagation: si source est Inhibitory → signal *= -1 (ou gain négatif dédié)
```

#### Pilier 2 : STDP (Spike-Timing Dependent Plasticity)

La plasticité doit prendre en compte **l'ordre d'activation** :
- Si le pré-synaptique (source) s'active *avant* le post-synaptique (cible) → renforcement (LTP).
- Si le post s'active *avant* le pré → affaiblissement (LTD).

Implémentation minimale :
```
Chaque arête stocke : last_pre_tick, last_post_tick
Δt = last_post_tick - last_pre_tick
Si Δt > 0 : Δw = +A_+ * exp(-Δt/τ_+)
Si Δt < 0 : Δw = -A_- * exp(Δt/τ_-)
```

Cela permet l'apprentissage de **séquences causales** : si A active systématiquement B, l'arête A→B est renforcée (mais pas B→A).

#### Pilier 3 : PID indirect

Le PID ne doit plus injecter d'activation directement. À la place :
```
Si activité zone < target :
    θ_zone -= δ    (abaisser le seuil → faciliter l'activation)
    g_zone += δ    (augmenter le gain de propagation)
Si activité zone > target :
    θ_zone += δ
    g_zone -= δ
```

Le réseau doit être actif **par sa propre dynamique de propagation**, le PID ne faisant qu'ajuster les conditions-cadre.

### 4.3 Résultats attendus de V3

Avec ces trois piliers, on devrait observer :

| Phénomène | Mécanisme |
|---|---|
| **Oscillations gamma émergentes** | Boucles E→I→E récurrentes |
| **Assemblées proto-stables** | Excitation réciproque + inhibition latérale |
| **Compétition entre patterns** | Inhibition latérale → winner-take-all partiel |
| **Apprentissage de séquences** | STDP → arêtes directionnelles causales |
| **Activité auto-entretenue** | Propagation récurrente + PID indirect comme filet |
| **Consolidation sélective** | Contraste spatial (inhibition) → seuls les chemins forts survivent |

---

## 5. Horizon V4–V6

### V4 — Spécialisation neuronale

Avec E/I en place, introduire des sous-types :
- **Neurones mémoire** : trace longue, seuil bas, plasticité forte.
- **Neurones relay** : rapides, fatigue faible, pas de trace.
- **Neurones intégrateurs** : seuil élevé, beaucoup de connexions entrantes, activité lente.
- **Neurones action** : seuil très élevé, activation rare mais forte propagation.

La diversité crée des **rôles fonctionnels** émergents : certains neurones deviennent naturellement des hubs, d'autres des capteurs, d'autres des intégrateurs.

### V5 — Mémoire multi-système

- **Mémoire de travail** : assemblées maintenues par excitation réciproque (requiert V3 — assemblées).
- **Mémoire long terme** : chemins consolidés (V2, mais sélectivement grâce à V3).
- **Mémoire procédurale** : séquences STDP consolidées.
- **Replay** : réactivation spontanée de patterns pendant les phases calmes.

### V6 — Assemblées dynamiques et proto-raisonnement

- Formation, maintien, et dissolution d'assemblées.
- Chaînage : une assemblée en active une autre.
- Proto-raisonnement : le système "teste" des assemblées internes.

---

## 6. Les risques de la roadmap

### 6.1 Risque de sur-ingénierie PID

Le PID marche trop bien. Le risque est de continuer à ajouter des contrôleurs PID plus sophistiqués (PID hiérarchique, PID adaptatif, etc.) au lieu de construire un réseau qui s'auto-organise. Le PID devrait **diminuer** en importance à chaque version, pas augmenter.

### 6.2 Risque d'explosion paramétrique

V2 a déjà ~30 paramètres configurables. Chaque version en ajoute. Le risque est de perdre la compréhension du système dans un espace paramétrique trop vaste. 

**Mitigation** : chaque version doit documenter les plages de stabilité (valeurs min/max qui fonctionnent) et les interactions entre paramètres.

### 6.3 Risque du "saut V7"

La roadmap actuelle prévoit V7 = "symbolisation émergente" et V8 = "agent textuel". Le gouffre entre V6 (assemblées dynamiques) et V7 est immense. Aucun système connectionniste n'a démontré de symbolisation émergente de façon convaincante.

**Mitigation** : poser un **gate expérimental** après V6 :
- Les assemblées émergent-elles spontanément ? Sont-elles stables ? Compétitives ?
- Peut-on les observer, les mesurer, les prédire ?
- Si oui → V7. Si non → repenser l'approche (peut-être hybridation avec des mécanismes symboliques explicites).

### 6.4 Risque de scalabilité

La roadmap est silencieuse sur l'infrastructure. À 50k nœuds le système est intéressant mais trop petit pour des hiérarchies riches. V3 devrait fonctionner à 500k–1M nœuds pour que les zones hiérarchiques aient un sens.

**Mitigation** : prévoir un sprint GPU (wgpu-compute ou CUDA) entre V3 et V4. Les opérations critiques (propagation, PID, plasticité) sont massivement parallélisables.

---

## 7. Métriques de succès par version

| Version | Métrique de succès | Critère de validation |
|---|---|---|
| V3 | Oscillations émergentes | Spectre de puissance montrant un pic de fréquence sans pacemaker |
| V3 | Assemblées proto-stables | Un groupe de nœuds co-actifs pendant >50 ticks sans stimulus externe |
| V3 | Séquences STDP | Stimulus A seul réactive le pattern B après entraînement A→B |
| V4 | Spécialisation fonctionnelle | Les types neuronaux montrent des rôles distincts mesurables |
| V5 | Working memory | Un pattern survit >200 ticks après disparition du stimulus, puis décline |
| V5 | Replay | Réactivation spontanée d'un pattern appris pendant une phase calme |
| V6 | Compétition d'assemblées | Deux stimuli simultanés → une seule assemblée survit |
| V6 | Chaînage | Assemblée A → assemblée B sans stimulus intermédiaire |

---

## 8. Résumé pour diriger la roadmap

### Ce qui est acquis et ne doit pas être cassé
- Activité auto-entretenue (PID comme filet de sécurité)
- Mode infini
- Déterminisme et reproductibilité
- Architecture modulaire (core/cli/viz)
- Rétro-compatibilité des configs

### Ce qui doit changer profondément
- Le PID doit devenir **indirect** (ajuster θ/g, pas a)
- Le réseau doit s'auto-entretenir par **propagation récurrente**, pas par injection
- L'**inhibition inter-neuronale** doit devenir un mécanisme de premier rang
- La plasticité doit devenir **temporellement causale** (STDP)
- La consolidation doit devenir **compétitive et sélective**

### L'ordre importe
```
1. Inhibition       → débloque compétition, contraste, oscillations émergentes
2. STDP             → débloque séquences, causalité, directionnalité
3. PID indirect     → débloque auto-entretien par propagation
4. GPU              → débloque la taille nécessaire pour les hiérarchies
```

Ce sont les **quatre investissements structurants** de V3. Tout le reste (spécialisation, mémoire, assemblées) en découle.
