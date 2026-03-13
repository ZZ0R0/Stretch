
# Cahier des charges — Version V1
## Substrat spatial 3D non-grille

## 1. Objet

La V1 constitue la première évolution majeure après la validation de la V0.
Elle vise à supprimer les deux limitations structurales identifiées :

- espace **2D**
- topologie **grille**

La V1 introduit un **substrat spatial 3D abstrait non-grille**, conçu pour supporter
les évolutions futures : morphogenèse, spécialisation neuronale, régions dynamiques
et croissance topologique.

---

# 2. Objectifs fonctionnels

La V1 doit permettre :

- représentation spatiale **3D**
- voisinages dynamiques
- propagation 3D anisotrope
- topologie non régulière
- indexation spatiale performante
- compatibilité avec croissance future

---

# 3. Domaine spatial V1

## 3.1 Position spatiale

Chaque neurone possède :

```
position : (x, y, z)
```

Contraintes :

- coordonnées flottantes
- domaine borné mais extensible
- distances euclidiennes utilisables

---

## 3.2 Indexation spatiale

La V1 doit abandonner les structures de grille.

Structures autorisées :

- k-nearest neighbors
- rayon spatial
- graphe spatial sparse
- index KD-tree
- spatial hash

Objectif :

```
requête voisinage < O(log n)
```

---

## 3.3 Voisinage dynamique

Deux types de connexions doivent exister :

### connexions locales

voisins dans un rayon spatial

### connexions longues

liaisons créées par plasticité

---

# 4. Propagation V1

La propagation doit fonctionner dans l'espace 3D.

Influence reçue par un neurone i :

```
phi_i = Σ_j activation_j * conductance_ji * kernel(distance(i,j))
```

Kernel minimal :

```
exp(-lambda * distance)
```

Propriétés requises :

- propagation isotrope possible
- anisotropie possible
- dissipation temporelle

---

# 4b. Dissipation V1

## 4b.1 Potentiel de repos (activation minimale)

En V0, l'activation décroît vers exactement 0.0. Cela provoque un **flatline universel** :
tous les nœuds convergent vers le même état mort, sans variabilité résiduelle.

La V1 introduit un **potentiel de repos** (`activation_min`) :

```
activation = max(activation * (1 - decay_rate), activation_min)
```

Objectif : maintenir un bruit de fond minimal, empêcher le flatline absolu,
et rendre la reprise d'activité plus naturelle.

## 4b.2 Decay aléatoire progressif (jitter)

En V0, le taux de decay est identique pour tous les nœuds à chaque tick.
Cela crée une dissipation artificiellement synchrone.

La V1 introduit un **jitter stochastique** sur le taux de decay :

```
effective_decay = base_decay * (1 + uniform(-jitter, +jitter))
```

Propriétés :

- chaque nœud reçoit un taux légèrement différent à chaque tick
- le jitter est borné et reproductible (même seed → mêmes résultats)
- l'amplitude du jitter est configurable (ex: 0.15 = ±15%)
- cela brise la synchronie artificielle de la dissipation
- favorise l'émergence de variabilité dans les dynamiques locales

---

# 5. Structures de données

## Node

```
id
position
activation
threshold
fatigue
memory_trace
excitability
inhibition
```

## Edge

```
source
target
conductance
coactivity_trace
plasticity
decay
distance
```

---

# 6. Performances minimales

La V1 doit supporter :

```
10k neurones minimum
```
avec simulation interactive.

Objectif cible :

```
100k neurones
```

---

# 7. Instrumentation

La V1 doit produire :

- heatmap 3D activation
- distribution distances propagation
- corridors 3D
- métriques énergie globale

---

# 8. Critères d'acceptation

La V1 est validée si :

- propagation stable en 3D
- corridors émergent aussi en 3D
- aucune dépendance à une grille
- performances acceptables
