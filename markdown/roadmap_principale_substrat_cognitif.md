# Nouvelle roadmap — à partir de V2

*(adaptée aux résultats et instabilités V1)*

## État réel après V1

Le système possède maintenant :

* substrat **3D non-grid**
* graphe spatial scalable **50k nœuds**
* propagation dissipative stable
* plasticité locale
* mémoire courte
* formalisation mathématique complète
* reproductibilité déterministe

Mais il possède aussi des **limitations structurelles critiques** :

1. aucune **activité auto-entretenue**
2. aucune **oscillation stable**
3. mémoire structurelle **éphémère**
4. aucune **régulation locale**
5. nœuds **uniformes**
6. dynamique purement **stimulus → propagation → extinction**

Donc la roadmap doit maintenant viser :

```
substrat dynamique passif
→ substrat auto-régulé
→ substrat spécialisé
→ substrat mémoriel
→ substrat cognitif
→ agent textuel
```

---

# V2 — Régulation locale et activité endogène

## Objectif

Transformer le système d’un **réseau purement réactif** en un système **capable de maintenir une activité dynamique stable**.

Sans cette étape, aucune cognition n’est possible.

Les instabilités V1 montrent clairement que **l’équilibre actif n’existe pas** et que toute activité s’éteint .

---

## Transformations majeures

### 1. Types de neurones

Introduction de **trois classes fondamentales** :

```
standard_neuron
control_neuron
pacemaker_neuron
```

---

### 2. Neurones de contrôle (régulateurs PID)

Chaque zone possède un **neurone de contrôle**.

Il mesure :

```
activité moyenne zone
```

et injecte une correction :

```
u = Kp * erreur + Ki * intégrale + Kd * dérivée
```

où :

```
erreur = a_target - a_zone
```

Capacités obtenues :

* activité auto-entretenue
* oscillations contrôlées
* stabilisation locale

---

### 3. Partitionnement spatial

L’espace est divisé en **zones dynamiques** :

options :

```
Voronoï
rayon fixe
clustering spatial
```

Chaque zone possède :

```
1 neurone de contrôle
N neurones standards
```

---

### 4. Cycle enrichi d’un tick

Nouvelle séquence :

```
1 mesure activité zones
2 régulation PID
3 stimulus externe
4 propagation
5 dissipation
6 plasticité
```

---

### 5. Mémoire structurelle persistante

Solution au problème :

```
conductance → w_min
```

Nouveau mécanisme :

```
consolidation threshold
```

si

```
w > w_consolidation pendant T ticks
```

alors :

```
decay désactivé
```

---

## Résultat attendu V2

Le système peut produire :

```
activité auto-entretenue
oscillations locales
ondes périodiques
mémoire structurelle durable
```

---

# V3 — Architecture régionale et hiérarchie dynamique

## Objectif

Introduire une **organisation macroscopique** du réseau.

Le cerveau n’est pas un graphe homogène.

---

## Transformations

### 1. Hiérarchie de zones

```
micro-zones
meso-zones
macro-zones
```

Chaque niveau possède ses régulateurs.

---

### 2. Couplage entre zones

Les neurones de contrôle peuvent se **synchroniser**.

Exemples :

```
phase locking
battements
ondes globales
```

---

### 3. Régulation métabolique

Chaque zone possède :

```
budget d'activité
```

si dépassé :

```
inhibition régionale
```

---

### 4. Flux d'information

Introduction de :

```
zones de propagation rapide
zones de propagation lente
zones mémoire
zones oscillatoires
```

---

## Résultat attendu V3

Apparition de :

```
rythmes
patterns globaux
coordination multi-zones
```

---

# V4 — Spécialisation neuronale

## Objectif

Créer une **division fonctionnelle réelle**.

---

## Types introduits

```
memory neurons
relay neurons
action neurons
inhibitory neurons
control neurons
pacemaker neurons
```

---

## Différences possibles

Chaque type possède :

```
dynamique différente
plasticité différente
fatigue différente
temps de réponse différent
```

---

## Motifs émergents

Le système peut former :

```
assemblées neuronales
circuits récurrents
boucles de contrôle
corridors spécialisés
```

---

# V5 — Architecture mémoire

Objectif :

transformer les traces V1 en **mémoire réelle**.

---

## Trois mémoires

### mémoire de travail

assemblées actives temporaires

---

### mémoire long terme

chemins consolidés

---

### mémoire procédurale

séquences automatisées

---

## Consolidation

Processus :

```
activité
→ renforcement
→ consolidation
→ protection contre decay
```

---

## Replay

Pendant phases calmes :

```
réactivation motifs
renforcement différé
```

---

# V6 — Assemblées dynamiques et proto-raisonnement

Objectif :

permettre au système de **manipuler des états internes**.

---

## Assemblées neuronales

groupes temporaires de neurones représentant :

```
concept
état
objet interne
```

---

## Compétition

plusieurs assemblées peuvent :

```
entrer en compétition
s'inhiber
fusionner
```

---

## Boucles internes

le système peut :

```
tester un motif
observer résultat
ajuster dynamique
```

C’est la première étape vers le **raisonnement interne**.

---

# V7 — Symbolisation émergente

Objectif :

faire émerger une **représentation discrète**.

---

## Tokenisation interne

motifs dynamiques stables deviennent :

```
symboles internes
```

---

## Relations symboliques

possibilité d'apprendre :

```
séquences
associations
structures
```

---

## Interface externe

entrée :

```
texte simple
```

conversion :

```
texte → activation motifs
```

sortie :

```
motifs → tokens
```

---

# V8 — Agent textuel émergent

Objectif :

créer un système capable de :

```
micro conversation
mémoire courte
cohérence locale
réponses simples
```

---

## Architecture

boucle :

```
texte
→ encodage
→ assemblées internes
→ sélection réponse
→ génération texte
```

---

## Capacités minimales

le système peut :

```
maintenir contexte court
répondre à questions simples
apprendre patrons textuels
```

---

# Architecture finale de la roadmap

```
V0  substrat dynamique simple
V1  espace 3D non-grid

V2  régulation locale + activité endogène
V3  architecture régionale hiérarchique
V4  spécialisation neuronale
V5  mémoire multi-système
V6  assemblées dynamiques
V7  symbolisation
V8  agent textuel
```

---

# Le vrai point critique

La **version la plus importante de toute la roadmap** est :

```
V2
```

Car elle transforme :

```
réseau passif
→ système auto-régulé
```

Sans V2 :

```
pas d'activité stable
pas de cognition
pas de mémoire durable
pas d'agent
```