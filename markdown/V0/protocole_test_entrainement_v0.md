# Protocole de test et validation — Système V0 de dynamique spatio-temporelle cognitive

## 1. Objet

Ce document définit le **plan de test d’entraînement et de validation** du système V0.

Le but n’est pas de tester une performance de classification ou de langage, mais de vérifier que le système :

- propage une activité de manière spatio-temporelle ;
- dissipe cette activité de façon stable ;
- conserve des traces ;
- déforme progressivement ses chemins de propagation ;
- fait émerger des trajectoires préférentielles ;
- reste dans un régime exploitable.

En d’autres termes, on ne teste pas “l’intelligence” mais la **validité du substrat dynamique**.

---

## 2. Questions à trancher

Le protocole doit permettre de répondre aux questions suivantes :

1. Le système réagit-il localement à une stimulation ?
2. La propagation observée est-elle cohérente avec la topologie ?
3. L’activité se dissipe-t-elle correctement ?
4. Le système conserve-t-il une trace des activations passées ?
5. Les répétitions modifient-elles les trajectoires futures ?
6. Des chemins préférentiels émergent-ils réellement ?
7. Le système reste-t-il stable dans le temps ?
8. Le comportement observé est-il robuste aux petites variations de paramètres ?

---

## 3. Définition du succès V0

Le système V0 sera considéré comme valide si les trois propriétés suivantes sont démontrées simultanément :

### 3.1 Propagation
Une stimulation locale doit produire une propagation mesurable, spatialement structurée, non triviale.

### 3.2 Mémorisation dynamique
Des répétitions doivent modifier durablement au moins une partie du milieu.

### 3.3 Réutilisation
Après entraînement, un stimulus similaire doit emprunter plus facilement ou plus rapidement certains trajets que lors de l’état initial.

---

## 4. Stratégie générale de test

Le protocole est composé de 4 phases :

1. **Calibration**  
   Trouver une zone de paramètres stable.
2. **Apprentissage contrôlé**  
   Appliquer des séquences répétées de stimulation.
3. **Probe / rappel**  
   Tester si la réponse a changé après entraînement.
4. **Analyse**  
   Mesurer l’émergence de chemins préférentiels et la stabilité.

---

## 5. Métriques à mesurer

## 5.1 Métriques de base

### 5.1.1 Nombre de nœuds activés
Mesure :
```text
N_active(t)
```

Utilité :
- détecter propagation nulle ;
- détecter emballement global ;
- comparer avant / après entraînement.

### 5.1.2 Énergie globale du système
Mesure :
```text
E(t) = somme_i activation_i(t)
```

Utilité :
- suivre l’intensité d’activité ;
- détecter saturation ou extinction trop rapide.

### 5.1.3 Durée de réverbération
Mesure :
- nombre de ticks entre la stimulation initiale et le retour sous un seuil global.

Utilité :
- mesurer persistance ;
- comparer avant/après.

### 5.1.4 Rayon effectif de propagation
Mesure :
- distance moyenne ou maximale atteinte par l’activité à partir de la source.

Utilité :
- vérifier la propagation spatiale.

---

## 5.2 Métriques de structure

### 5.2.1 Usage des chemins
Mesure :
```text
usage_ij = nombre de fois où la liaison (i,j) participe à une propagation
```

Utilité :
- détecter les routes récurrentes ;
- construire une carte des corridors émergents.

### 5.2.2 Distribution des conductances
Mesure :
- histogramme des `conductance_ij`.

Utilité :
- vérifier si l’apprentissage déforme réellement le milieu.

### 5.2.3 Carte des traces locales
Mesure :
- distribution spatiale de `memory_trace_i`.

Utilité :
- vérifier où la mémoire se concentre.

### 5.2.4 Entropie des trajets
Mesure :
- dispersion des chemins utilisés par une même famille de stimuli.

Utilité :
- vérifier si le système se structure.
- une baisse d’entropie peut indiquer l’émergence de corridors privilégiés.

---

## 5.3 Métriques de performance du substrat

### 5.3.1 Temps d’accès à une zone cible
Mesure :
- nombre de ticks pour qu’une région cible atteigne un seuil d’activation après stimulation.

Utilité :
- mesurer si l’apprentissage facilite l’atteinte d’un motif.

### 5.3.2 Coût énergétique d’atteinte
Mesure :
- énergie cumulée dépensée avant activation d’une zone cible.

Utilité :
- détecter si une routine devient moins coûteuse.

### 5.3.3 Taux de réussite d’activation cible
Mesure :
- proportion d’essais où une région cible est effectivement activée.

Utilité :
- mesurer la fiabilité des chemins appris.

---

## 5.4 Métriques de stabilité

### 5.4.1 Taux d’emballement
Mesure :
- proportion de simulations où l’activité diverge ou reste saturée au-delà d’un seuil.

### 5.4.2 Taux d’extinction triviale
Mesure :
- proportion de simulations où la propagation meurt en moins de `k` ticks.

### 5.4.3 Sensibilité paramétrique
Mesure :
- variation des métriques lorsque les paramètres changent légèrement.

Utilité :
- évaluer la robustesse.

---

## 6. Ce qu’il faut regarder visuellement

Le système doit être inspecté visuellement, pas seulement numériquement.

## 6.1 Cartes temporelles
À observer :
- fronts d’activation ;
- vitesse d’expansion ;
- dissipation ;
- zones récurrentes.

## 6.2 Heatmaps de traces
À observer :
- concentration progressive de mémoire ;
- zones mortes ;
- zones saturées.

## 6.3 Heatmaps de conductance
À observer :
- apparition de couloirs ;
- renforcement local ;
- déformation du milieu.

## 6.4 Courbes temporelles
À tracer :
- énergie globale ;
- nombre de nœuds actifs ;
- durée de réverbération ;
- réussite cible.

---

## 7. Jeux de tests à exécuter

## 7.1 Test A — Sanity check de propagation

### But
Vérifier que le système propage localement sans apprentissage.

### Procédure
- initialiser un graphe simple ;
- appliquer un stimulus unique à un nœud central ;
- observer la propagation pendant `T` ticks.

### Attendu
- activation de la source ;
- propagation vers le voisinage ;
- décroissance ;
- retour au repos.

### Échec
- pas de propagation ;
- propagation instantanément globale ;
- activité qui ne s’éteint jamais.

---

## 7.2 Test B — Stabilité

### But
Trouver une zone de paramètres exploitable.

### Procédure
Faire varier :
- dissipation ;
- fatigue ;
- gain de propagation ;
- inhibition.

### Attendu
Identifier une zone intermédiaire où :
- l’activité survit quelques ticks ;
- elle se propage ;
- elle s’éteint ensuite.

### Résultat attendu
Une carte de phase qualitative :
- mort ;
- stable exploitable ;
- saturation.

---

## 7.3 Test C — Répétition d’un trajet

### But
Vérifier qu’un trajet répété laisse une empreinte.

### Procédure
- choisir un point source `S` ;
- choisir une région cible `C` ;
- stimuler `S` plusieurs fois selon le même protocole ;
- mesurer l’évolution des chemins empruntés.

### Attendu
Au fil des répétitions :
- certaines liaisons voient leur usage augmenter ;
- certaines conductances locales augmentent ;
- le temps d’accès à `C` diminue ou se stabilise favorablement ;
- le coût énergétique baisse ou se réorganise.

---

## 7.4 Test D — Probe avant / après entraînement

### But
Vérifier que l’entraînement change réellement la dynamique.

### Procédure
1. mesurer la réponse à un stimulus de test avant entraînement ;
2. entraîner le système ;
3. rejouer le même stimulus ;
4. comparer.

### Mesures clés
- temps d’accès ;
- énergie cumulée ;
- entropie des trajets ;
- distribution des activations ;
- conductances sollicitées.

### Attendu
Une différence nette et reproductible avant/après.

---

## 7.5 Test E — Généralisation locale

### But
Vérifier si le système apprend un corridor et pas seulement un point exact.

### Procédure
- entraîner depuis une source `S` ;
- tester ensuite des sources proches de `S` ;
- observer si elles bénéficient partiellement du chemin appris.

### Attendu
- amélioration locale autour du trajet appris ;
- pas seulement sur le nœud exact.

### Intérêt
C’est un bon indicateur d’émergence d’une structure spatiale utile.

---

## 7.6 Test F — Oubli

### But
Vérifier que le système n’est pas purement cumulatif.

### Procédure
- entraîner un corridor ;
- arrêter les stimulations ;
- observer la décroissance des traces et conductances sur une longue fenêtre.

### Attendu
- décroissance partielle ou lente ;
- conservation sélective si le mécanisme le prévoit ;
- pas de mémoire infinie artificielle partout.

---

## 8. Séquence d’entraînement recommandée

## 8.1 Phase 1 — Baseline
- aucune plasticité ou plasticité minimale ;
- mesurer le comportement initial.

## 8.2 Phase 2 — Entraînement
- répéter `N` fois un même schéma de stimulation ;
- espacer les épisodes pour laisser la dissipation agir.

## 8.3 Phase 3 — Probe
- exécuter des stimuli tests ;
- geler les paramètres ;
- comparer avant/après.

## 8.4 Phase 4 — Robustesse
- répéter avec plusieurs graines ;
- répéter avec petites variations de paramètres.

---

## 9. Indicateurs de bon fonctionnement

Le système fonctionne bien si l’on observe :

1. **une propagation locale nette** ;
2. **une dissipation contrôlée** ;
3. **une accumulation de traces non uniforme** ;
4. **une déformation graduelle des conductances** ;
5. **une amélioration du rappel ou de l’accès cible** ;
6. **une stabilité qualitative sur plusieurs runs**.

---

## 10. Indicateurs d’échec

Le système ne fonctionne pas correctement si l’on observe :

### 10.1 Système mort
- très peu de nœuds activés ;
- extinction immédiate ;
- aucune trace durable.

### 10.2 Système saturé
- activation globale quasi permanente ;
- perte de localisation ;
- aucune structure lisible.

### 10.3 Faux apprentissage
- conductances qui montent partout ;
- gain global non sélectif ;
- absence de corridors ou motifs localisés.

### 10.4 Apprentissage instable
- résultat très différent à chaque run ;
- trajectoires incohérentes ;
- forte sensibilité au moindre changement.

---

## 11. Matrice d’évaluation synthétique

## 11.1 Critères

### Propagation
- note 0 : pas de propagation
- note 1 : propagation triviale
- note 2 : propagation locale correcte
- note 3 : propagation riche et contrôlée

### Stabilité
- note 0 : mort ou explosion
- note 1 : instable
- note 2 : stable exploitable
- note 3 : stable robuste

### Mémoire
- note 0 : aucune trace
- note 1 : traces faibles sans effet
- note 2 : traces modifiant la réponse
- note 3 : traces structurantes claires

### Plasticité
- note 0 : aucune évolution
- note 1 : évolution diffuse non sélective
- note 2 : corridors émergents
- note 3 : corridors robustes et mesurables

### Réutilisation
- note 0 : aucune différence avant/après
- note 1 : différence marginale
- note 2 : amélioration claire
- note 3 : amélioration robuste et reproductible

---

## 12. Expériences minimales à archiver

Pour chaque campagne de test, archiver :

- configuration complète ;
- graine aléatoire ;
- topologie initiale ;
- courbes temporelles ;
- heatmaps finales ;
- tableau des métriques ;
- comparaison avant/après ;
- commentaire d’interprétation.

---

## 13. Conclusion

Le système V0 ne doit pas être évalué comme un modèle de prédiction, mais comme un **milieu d’apprentissage dynamique**.

La bonne question n’est pas :

> “Est-ce qu’il répond intelligemment ?”

La bonne question est :

> “Est-ce qu’il développe des chemins spatio-temporels stables, sélectifs, réutilisables et mesurables sous l’effet de répétitions ?”

Si la réponse est oui, la V0 est valide et peut servir de base à une V1 plus riche.
