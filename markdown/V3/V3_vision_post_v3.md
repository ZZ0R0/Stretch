# Vision post-V3 — Direction et architecture future

Ce document est destiné à être fourni à un LLM pour contextualiser la suite de la roadmap. Il résume **ce que le système est devenu**, **ce qui manque fondamentalement**, et **la direction architecturale des versions futures**.

---

## 1. Ce que Stretch est aujourd'hui (fin V3)

### 1.1 Nature du système

Stretch est un **substrat de dynamiques spatio-temporelles** simulé en Rust : un graphe spatial 3D de 500 000 nœuds (80% excitateurs, 20% inhibiteurs) connectés par ~5 millions d'arêtes pondérées. L'activité se propage de façon signée (E positif, I négatif), se dissipe, et s'adapte par plasticité Hebbienne + STDP. Le PID indirect ajuste les conditions-cadre (seuil, gain) sans injecter d'activation. Le budget synaptique crée une compétition entre connexions sortantes.

### 1.2 Ce que V3 sait faire

| Capacité | Qualité |
|---|---|
| Activité auto-entretenue (PID indirect) | ✅ Stable, naturelle |
| Inhibition inter-neuronale (E/I) | ✅ Contraste spatial, compétition locale |
| Propagation signée | ✅ E > 0, I < 0 |
| Plasticité temporellement causale (STDP) | ✅ LTP/LTD, directionnalité |
| Budget synaptique compétitif | ✅ Consolidation sélective |
| Parallélisme 16 threads | ✅ rayon auto-detect |
| Oscillations pacemaker | ✅ Persistantes |
| 500k nœuds | ✅ Fonctionnel |

### 1.3 Ce que V3 **ne sait pas faire**

| Incapacité | Cause racine |
|---|---|
| Apprentissage guidé par un objectif | Pas de signal de récompense |
| Modulation de la plasticité par le contexte | Pas de voies dopaminergiques |
| Recevoir des données structurées | Pas d'interface d'entrée |
| Produire des sorties interprétables | Pas d'interface de sortie |
| Maintien intentionnel d'un pattern | Pas de mémoire de travail active |
| Prédiction temporelle | Pas de réactivation anticipatoire |
| Oscillations émergentes certaines | Non validé formellement |

---

## 2. Le verrou fondamental : l'absence de dopamine

### 2.1 Pourquoi c'est critique

Le cerveau utilise la dopamine comme **signal d'erreur de prédiction de récompense** (Schultz, 1997). Ce signal module la plasticité :

- Récompense inattendue → bouffée de dopamine → LTP renforcée sur les synapses récemment actives → le comportement qui a mené à la récompense est consolidé.
- Récompense attendue absente → creux de dopamine → LTD → le comportement qui a échoué est affaibli.
- Récompense attendue reçue normalement → pas de modulation → stabilité.

Sans ce mécanisme, le réseau Stretch apprend de manière **purement locale et non guidée** : la STDP renforce les séquences temporelles fréquentes, mais ne distingue pas les séquences utiles des séquences inutiles. C'est de l'apprentissage Hebbien enrichi, pas de l'apprentissage par renforcement.

### 2.2 Ce que la dopamine débloque

| Capacité | Mécanisme dopaminergique requis |
|---|---|
| Apprentissage par renforcement | δ = r - V(s) module la plasticité |
| Consolidation sélective | Seules les arêtes actives pendant un signal de récompense sont consolidées |
| Attention endogène | La saillance (prédiction de récompense) augmente le gain local |
| Motivation / exploration | La dopamine tonique module le seuil de déclenchement global |
| Mémoire de travail | La dopamine préfrontale maintient les assemblées actives |

**Constat** : sans voies dopaminergiques, le système ne peut pas fonctionner comme un vrai cerveau fonctionnerait. C'est une brique structurelle manquante, pas un ajout optionnel.

### 2.3 Architecture dopaminergique minimale

Implémentation proposée pour les prochaines versions :

```
DopamineSource {
    nucleus: Vec<usize>,           // nœuds "VTA/SNc" (source de DA)
    targets: Vec<ZoneId>,          // zones cibles (régions recevant la DA)
    tonic_level: f64,              // niveau basal de DA
    phasic_burst: f64,             // signal positif (récompense inattendue)
    phasic_dip: f64,               // signal négatif (récompense attendue absente)
}

Effet sur la plasticité :
    Δw_STDP *= dopamine_level      // la DA module l'amplitude STDP
    consolidation_gate = dopamine_level > da_threshold

Effet sur l'activation :
    gain_zone *= (1 + k_da * dopamine_level)
    threshold_zone -= k_da_theta * dopamine_level
```

La dopamine agirait comme un **troisième canal orthogonal** aux signaux excitateurs et inhibiteurs : elle ne transporte pas d'information spatio-temporelle mais **module les règles d'apprentissage** et le gain de propagation.

---

## 3. Architecture d'entrée — Planification progressive

### 3.1 Problème actuel

Le système reçoit des stimuli via injection directe d'activation sur des nœuds spécifiques à des ticks fixés dans le fichier de config. C'est :
- arbitraire (les nœuds stimulés n'ont pas de signification) ;
- statique (les patterns sont hard-codés, pas adaptés à un flux d'entrée) ;
- non structuré (pas d'encodage de l'information).

### 3.2 Feuille de route des entrées

#### V4 — Entrées structurées simples

- **Zone d'entrée dédiée** : réserver une portion du graphe (~5–10% des nœuds) comme couche d'entrée.
- **Encodage topographique** : mapper un vecteur d'entrée $\mathbf{x} \in \mathbb{R}^d$ sur les nœuds d'entrée via une projection spatiale.
- **Encodage temporel** : convertir un signal scalaire en pattern d'activation temporel (rate coding ou temporal coding).
- Cas d'usage : discriminer 2–3 patterns d'entrée distincts (ex : A vs B vs C encodés comme 3 régions spatiales de la zone d'entrée).

#### V5 — Encodage sensoriel

- **Tokenisation** : convertir un flux de tokens (texte, symboles) en séquences de patterns d'activation.
- **Projection apprise** : les poids de la couche d'entrée sont adaptés par STDP (le réseau apprend à encoder).
- **Fenêtre temporelle** : l'entrée est un flux continu, pas des stimuli ponctuels.

#### V6 — Interface sensorielle riche

- **Multi-modal** : plusieurs zones d'entrée pour différents types de signaux.
- **Feedback** : les zones d'entrée reçoivent aussi des signaux top-down du réseau (attentes, prédictions).
- **Attention entrante** : modulation du gain des zones d'entrée par la dopamine ou un mécanisme d'attention.

### 3.3 Contrainte architecturale

L'interface d'entrée ne doit **pas** ressembler à une couche de réseau de neurones classique (pas de poids de projection appris par backprop). Elle doit être un **sous-graphe** du substrat, avec les mêmes règles de propagation et de plasticité, mais dont les nœuds reçoivent des stimuli externes structurés au lieu d'injections ad hoc.

---

## 4. Architecture de sortie — Planification progressive

### 4.1 Problème actuel

Le système ne produit aucune sortie interprétable. On peut observer l'état interne (activation de chaque nœud, métriques globales), mais il n'y a pas de mécanisme de lecture de l'état pour prendre des décisions ou communiquer.

### 4.2 Feuille de route des sorties

#### V4 — Lecture d'état minimale

- **Zone de sortie dédiée** : réserver une portion du graphe (~5–10% des nœuds) comme couche de sortie.
- **Décodage par population** : lire l'activité moyenne de sous-régions de la zone de sortie pour décoder une "décision" (ex : quelle sous-région est la plus active → choix A, B ou C).
- **Seuil de décision** : une décision est émise quand l'activité d'une sous-région de sortie dépasse un seuil pendant N ticks.

#### V5 — Sortie séquentielle

- **Émission temporelle** : la zone de sortie produit une séquence de patterns (pas un vecteur unique).
- **Action motrice** : la sortie déclenche une action dans un environnement simulé.
- **Boucle** : l'action modifie l'entrée → rétroaction sensori-motrice.

#### V6 — Interface symbolique

- **Readout appris** : le réseau apprend quels patterns de sortie correspondent à quelles catégories.
- **Proto-langage** : la sortie peut correspondre à des tokens ou symboles discrets.

### 4.3 Contrainte architecturale

Comme pour l'entrée, la zone de sortie doit être un **sous-graphe standard** du substrat, pas un décodeur externe. La lecture est une observation de l'activité de ce sous-graphe, pas une opération algébrique sur les poids.

---

## 5. Architecture de récompense — Planification progressive

### 5.1 Problème actuel

Le système n'a aucun signal de récompense. L'apprentissage est purement Hebbien/STDP : il capture les corrélations et la causalité temporelle, mais ne peut pas distinguer un bon comportement d'un mauvais.

### 5.2 Feuille de route de la récompense

#### V4 — Signal de récompense externe simple

- **Reward signal** : un scalaire $r(t) \in [-1, +1]$ injecté de l'extérieur à certains ticks.
- **Trace d'éligibilité** : chaque arête accumule une trace d'éligibilité $e_{ij}$ qui décroît exponentiellement :

$$e_{ij}(t+1) = \gamma_e \cdot e_{ij}(t) + \Delta w_{\text{STDP}}(t)$$

- **Modulation dopaminergique** : le reward module la plasticité effective :

$$\Delta w_{\text{eff}} = r(t) \times e_{ij}(t)$$

- Cela signifie que les arêtes récemment renforcées par STDP sont consolidées si une récompense arrive dans la fenêtre d'éligibilité, et affaiblies si une punition arrive.

#### V5 — Prédiction de récompense (TD learning)

- **Valeur d'état** : certains nœuds apprennent à prédire la récompense future $V(s)$.
- **Erreur de prédiction** : $\delta = r + \gamma V(s') - V(s)$ — la dopamine est ce signal $\delta$.
- **Modulation adaptative** : la dopamine ne dépend plus du reward brut mais de la **surprise** (reward inattendu).

#### V6 — Récompense intrinsèque

- **Curiosité** : récompense pour la nouveauté (réduction de l'incertitude).
- **Consistance** : récompense pour la prédiction correcte.
- Le système développe une motivation endogène, pas seulement exogène.

---

## 6. Roadmap révisée V4–V11

La roadmap originale (V0–V11) nécessite une insertion des systèmes de récompense, d'entrée et de sortie plus tôt que prévu. La dopamine est un prérequis à toute forme d'apprentissage guidé.

```text
V0   substrat dynamique minimal                    ✅
V1   espace 3D non-grid                            ✅
V2   régulation locale et activité endogène         ✅
V3   inhibition, STDP, PID indirect                 ✅

V4   dopamine, reward, entrées/sorties minimales
     ├── voies dopaminergiques (VTA → zones cibles)
     ├── traces d'éligibilité
     ├── zones d'entrée et de sortie dédiées
     ├── reward externe simple
     └── profil STDP asymétrique

V5   sélectivité mémoire et compétition locale
     ├── gating événementiel de la consolidation
     ├── sous-types neuronaux riches
     ├── oscillations émergentes validées
     └── prédiction de récompense (TD)

V6   hiérarchie de zones et régulation multi-échelle
     ├── zones micro/méso/macro
     ├── flux ascendant/descendant
     ├── entrée séquentielle + encodage appris
     └── boucle sensori-motrice

V7   spécialisation neuronale riche
     ├── interneurones rapides/lents
     ├── cellules mémoire / relay / intégrateur
     └── récompense intrinsèque (curiosité)

V8   assemblées dynamiques et mémoire de travail
     ├── formation, maintien, dissolution d'assemblées
     ├── compétition inter-assemblées (winner-take-all)
     └── replay spontané

V9   chaînage, séquences et proto-raisonnement
     ├── assemblée A → assemblée B sans stimulus
     ├── proto-inférence par chaînage interne
     └── sortie symbolique

V10  symbolisation et interface discrète
     ├── émergence de catégories (si possible)
     ├── readout symbolique
     └── gate expérimental : symbolisation réelle ?

V11  agent textuel émergent minimal
     ├── tokenisation → activation → traitement → sortie token
     └── conversation minimale avec un objectif
```

### Changement principal par rapport à l'ancienne roadmap

La dopamine, les entrées et les sorties sont avancées de V7→V4. Sans ces briques, V5–V6 n'ont pas de fondation : on ne peut pas parler de compétition utile, de mémoire fonctionnelle, ou de hiérarchie si le système ne poursuit aucun objectif et ne reçoit aucune donnée structurée.

---

## 7. Les 4 axes transversaux

En parallèle des versions fonctionnelles, quatre axes transversaux doivent progresser continûment :

### 7.1 Performance

```
V4 : GPU compute (wgpu) pour propagation + plasticité → 500k–1M temps réel
V5 : instanced rendering pour viz → 1M nœuds visualisables
V6 : multi-GPU ou cluster si nécessaire
```

### 7.2 Instrumentation

```
V4 : métriques dopaminergiques, reward tracking, traces d'éligibilité
V5 : spectrogramme en temps réel, détection d'assemblées automatique
V6 : dashboards hiérarchiques par zone
```

### 7.3 Reproductibilité

```
V4 : enregistrement complet des sessions (config + seed + reward → même résultat)
V5 : format de sauvegarde/restauration d'état (checkpointing)
V6 : replay de sessions avec exploration what-if
```

### 7.4 Documentation

```
V4 : modèle mathématique complet incluant dopamine et I/O
V5 : protocoles d'évaluation d'apprentissage
V6 : articles/rapports de résultats
```

---

## 8. Risques de la roadmap

### 8.1 Risque du "saut dopamine"

La dopamine est un composant fondamental mais complexe. Le risque est de construire un système dopaminergique trop simplifié qui ne produit pas l'effet modulateur attendu, ou trop complexe qui ajoute une explosion paramétrique ingérable.

**Mitigation** : commencer par le cas le plus simple possible — un scalaire reward externe qui module l'amplitude STDP via une trace d'éligibilité. Valider que ça change effectivement les conductances apprises. Puis complexifier.

### 8.2 Risque d'empilement de mécanismes

Chaque version ajoute des mécanismes. V3 a déjà : PID, pacemakers, fatigue, inhibition, trace, excitabilité, STDP, budget, consolidation. V4 ajouterait : dopamine, éligibilité, zones I/O, reward. Le système risque de devenir un patchwork difficilement compréhensible.

**Mitigation** : chaque mécanisme doit être désactivable (enabled = false). Documenter les interactions entre mécanismes (quelle modulation affecte quoi). Tests d'ablation systématiques.

### 8.3 Risque du "gouffre V9–V10"

Le passage de proto-raisonnement par chaînage (V9) à symbolisation (V10) reste le point le plus incertain de toute la roadmap. Aucun système connectionniste n'a démontré de symbolisation émergente convaincante à ce jour.

**Mitigation** : gate dur après V9. Si le chaînage fonctionne mais que les assemblées ne se stabilisent pas en catégories discrètes → envisager une approche hybride (mécanismes symboliques explicites greffés sur le substrat connectionniste).

### 8.4 Risque de latence I/O

L'introduction d'interfaces d'entrée/sortie crée un nouveau goulot : le temps de conversion données → patterns → traitement → lecture est potentiellement plus long que le traitement lui-même.

**Mitigation** : benchmarker séparément le pipeline I/O. Commencer avec des entrées/sorties extrêmement simples (3 catégories, pas 1000).

---

## 9. Métriques de succès par version

| Version | Métrique de succès | Critère de validation |
|---|---|---|
| V4 | Reward change les poids | Comparaison STDP seule vs STDP+reward sur la même tâche |
| V4 | Zone d'entrée encode des patterns | Patterns distincts produisent des activations spatiales distinctes |
| V4 | Zone de sortie décode une décision | Accuracy > chance (>50% pour 2 choix) |
| V5 | Le système prédit la récompense | $V(s)$ converge vers la récompense réelle dans une tâche simple |
| V5 | Assemblées compétitives | 2 stimuli → 1 assemblée gagnante |
| V6 | Hiérarchie fonctionnelle | Zones hautes intègrent l'info de zones basses |
| V7 | Spécialisation mesurable | Sous-types montrent des rôles fonctionnels distincts |
| V8 | Mémoire de travail | Pattern maintenu >200 ticks sans stimulus, puis oublié |
| V9 | Chaînage causal | Assemblée A → B → C sans stimuli intermédiaires |
| V10 | Catégories discrètes | Clusters stables dans l'espace d'activation |

---

## 10. Résumé pour diriger la roadmap

### Ce qui est acquis et ne doit pas être cassé

- Activité auto-entretenue par propagation (PID indirect comme filet)
- Inhibition inter-neuronale E/I
- STDP temporellement causale
- Budget synaptique compétitif
- Parallélisme 16 threads
- Déterminisme et reproductibilité
- Architecture modulaire (core/cli/viz)

### Ce qui doit être construit dès V4

1. **Voies dopaminergiques** — le composant le plus critique manquant. Sans dopamine, le système ne peut pas apprendre de manière guidée, ne peut pas consolider sélectivement, et ne peut pas développer un comportement orienté vers un objectif. C'est la brique qui transforme un "substrat qui s'organise" en un "substrat qui apprend".

2. **Interfaces d'entrée et de sortie** — le système doit pouvoir recevoir des données structurées et produire des décisions interprétables. Sans I/O, c'est un aquarium : intéressant à observer mais incapable d'interagir.

3. **Architecture de récompense** — d'abord un reward externe simple, puis progressivement une prédiction de récompense (TD), puis une récompense intrinsèque. Chaque palier débloque un niveau d'autonomie.

### L'ordre importe

```
1. Dopamine                → débloque l'apprentissage guidé
2. Entrée structurée       → débloque la réception de données
3. Sortie interprétable    → débloque l'évaluation du comportement
4. Reward + éligibilité    → colle les trois ensemble
5. Performance GPU         → débloque l'échelle nécessaire
```

La V4 doit accomplir les quatre premiers points de manière minimale mais fonctionnelle. C'est le palier qui transforme Stretch de "simulateur de dynamiques" en "agent apprenant minimal".
