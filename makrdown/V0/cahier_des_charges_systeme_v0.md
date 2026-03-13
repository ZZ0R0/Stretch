# Cahier des charges — Système V0 de dynamique spatio-temporelle cognitive

## 1. Objet

Ce document définit la **V0** d’un système expérimental visant à reproduire, sous forme mathématique et informatique, des **dynamiques spatio-temporelles d’activation, de propagation, de dissipation, de renforcement et de structuration**, inspirées du fonctionnement cérébral, **sans** recourir :

- ni à un LLM ;
- ni à un réseau de neurones classique à poids appris par backpropagation ;
- ni à une simulation physico-chimique détaillée ;
- ni à un apprentissage statistique massif sur corpus.

L’objectif n’est pas de simuler la chimie réelle, mais de construire un **substrat dynamique abstrait**, situé au-dessus de la chimie et au-dessous du cognitif explicite, capable de faire émerger :

- des fronts de propagation ;
- des traces locales ;
- des chemins préférentiels ;
- des routines stabilisées ;
- des mécanismes élémentaires de mémoire et d’apprentissage.

---

## 2. Périmètre V0

La V0 est un **noyau de simulation**.

Elle doit permettre :

1. de représenter un espace discret de nœuds ;
2. de représenter des voisinages et des proximités ;
3. de propager une activation locale sous forme de front spatio-temporel ;
4. de dissiper cette activation dans le temps ;
5. de laisser une trace locale du passage de l’activité ;
6. de renforcer ou d’affaiblir localement les chemins parcourus ;
7. de mesurer si des trajectoires préférentielles émergent.

La V0 **ne doit pas** viser :

- la conscience ;
- le langage ;
- la perception réelle ;
- l’apprentissage de tâches industrielles ;
- la simulation fidèle du cerveau humain ;
- la distribution multi-machines ;
- l’optimisation hardware spécifique.

---

## 3. Vision de haut niveau

Le système doit être conçu comme un **milieu dynamique à mémoire**.

Une activation locale d’un nœud ne doit pas être modélisée comme une simple transmission scalaire, mais comme l’émission d’un **front d’influence** qui :

- se propage dans un espace topologique ;
- s’atténue avec le temps ;
- interagit avec les propriétés locales du milieu ;
- peut déclencher d’autres nœuds ;
- et laisse une empreinte qui modifie les futures propagations.

L’apprentissage de la V0 repose sur l’idée suivante :

```text
répétition de propagations
→ accumulation de traces
→ déformation locale du milieu
→ stabilisation de couloirs préférentiels
→ émergence de routines dynamiques
```

---

## 4. Exigences fonctionnelles

## 4.1 Domaine spatial

Le système doit gérer un domaine discret de type :

- graphe sparse ;
- ou grille 2D/3D abstraite ;
- ou ensemble de points muni d’une métrique.

### Exigences minimales
- Chaque nœud doit disposer d’un identifiant unique.
- Chaque nœud doit disposer d’une position abstraite ou d’un voisinage explicite.
- Le système doit supporter au minimum un graphe spatial sparse.
- La topologie initiale doit être chargeable depuis une configuration.

### Hors périmètre V0
- Géométrie biologique réelle.
- IRM / connectome réel.
- Maillage continu complexe.

---

## 4.2 État d’un nœud

Chaque nœud doit porter un état minimal composé de :

- `activation` : niveau d’activité courant ;
- `threshold` : seuil de déclenchement ;
- `fatigue` : état réfractaire / coût temporaire d’activation ;
- `memory_trace` : trace locale cumulée ;
- `excitability` : facilité actuelle à répondre ;
- `inhibition` : résistance locale à l’activation.

### Exigences
- Tous les états doivent être numériques et bornés.
- Les mises à jour doivent être déterministes à paramètres fixés.
- Le système doit permettre de geler ou d’inspecter l’état de n’importe quel nœud à tout instant.

---

## 4.3 État d’une liaison

Chaque liaison ou relation locale doit porter un état minimal :

- `conductance` : facilité de propagation ;
- `distance` : coût spatial/topologique ;
- `coactivity_trace` : historique local de co-activation ;
- `plasticity` : capacité de la liaison à évoluer ;
- `decay` : vitesse d’oubli / retour à la ligne de base.

### Exigences
- Les liaisons doivent être pondérées.
- La conductance doit être modifiable au cours de la simulation.
- La conductance ne doit jamais diverger hors des bornes configurées.

---

## 4.4 Propagation

Une activation locale doit engendrer un **front d’influence**.

La propagation doit dépendre au minimum de :
- l’intensité de la source ;
- la conductance locale ;
- la distance topologique ;
- l’état local du nœud cible ;
- la fatigue / inhibition du nœud cible.

### Exigences
- La propagation doit être simulée par pas de temps.
- Le système doit supporter un noyau de diffusion configurable.
- Le noyau minimal doit être décroissant avec la distance.
- La propagation doit être dissipative : sans stimulation, l’activité doit retomber.

### Exemples de noyaux autorisés
- exponentiel ;
- gaussien discret ;
- diffusion pondérée sur graphe.

### Exemples de noyaux exclus en V0
- solveurs physico-chimiques lourds ;
- PDE continues haute précision.

---

## 4.5 Déclenchement

Un nœud doit pouvoir passer à l’état actif si l’influence totale reçue dépasse un seuil corrigé de son état interne.

### Exigences
- Le déclenchement doit dépendre du seuil, de la fatigue et de l’excitabilité.
- Un nœud sur-sollicité doit devenir temporairement plus difficile à réactiver.
- Le système doit éviter les emballements globaux permanents.

---

## 4.6 Dissipation et stabilité

La V0 doit inclure des mécanismes de stabilisation.

### Exigences minimales
- décroissance spontanée de l’activation ;
- fatigue ou réfractarité ;
- inhibition locale ou globale ;
- bornage des variables ;
- contrôle de l’énergie totale du système.

### Objectif
Éviter les deux modes pathologiques :
- extinction immédiate ;
- saturation globale permanente.

---

## 4.7 Trace locale et mémoire

Le passage répété de l’activité doit laisser une trace locale.

### Exigences
- Chaque traversée doit pouvoir augmenter une trace locale.
- La trace doit décroître lentement si elle n’est plus réactivée.
- La trace doit influencer la propagation future.

### But
Permettre l’émergence de :
- mémoire de court terme par activité récurrente ;
- mémoire de moyen terme par accumulation de traces ;
- préfiguration d’une mémoire de long terme par stabilisation des chemins.

---

## 4.8 Plasticité

La structure de propagation doit pouvoir évoluer à partir de l’historique d’activité.

### Exigences
- Les liaisons fréquemment traversées doivent pouvoir se renforcer.
- Les liaisons peu utilisées doivent pouvoir s’affaiblir.
- La plasticité doit être lente relativement à l’activation instantanée.
- Les règles doivent être locales autant que possible.

### Hors périmètre V0
- création/suppression automatique de nœuds ;
- croissance topologique agressive ;
- reconfiguration globale complexe.

---

## 4.9 Instrumentation

La simulation doit être observable finement.

### Exigences
Le système doit exposer :
- état global à chaque tick ;
- état par nœud ;
- état par liaison ;
- nombre de nœuds actifs ;
- énergie totale ;
- chemins les plus traversés ;
- distribution des traces ;
- évolution des conductances.

---

## 5. Exigences non fonctionnelles

## 5.1 Lisibilité scientifique
- Le code doit être structuré pour permettre l’expérimentation.
- Les règles d’évolution doivent être explicites.
- Les paramètres doivent être centralisés et documentés.

## 5.2 Reproductibilité
- Toute simulation doit être rejouable avec une graine fixe.
- Les résultats doivent être exportables.
- Les expériences doivent être traçables via fichiers de configuration.

## 5.3 Modularité
La V0 doit séparer :
- domaine spatial ;
- dynamique des nœuds ;
- dynamique des liaisons ;
- propagation ;
- plasticité ;
- métriques ;
- visualisation.

## 5.4 Performance raisonnable
- La cible initiale est une simulation locale sur machine unique.
- La V0 doit pouvoir gérer au moins 1k à 100k nœuds selon la topologie choisie.
- La complexité doit rester compatible avec des expérimentations rapides.

## 5.5 Stabilité numérique
- Les variables doivent être bornées.
- Les règles doivent être robustes aux petites variations de paramètres.
- Le système doit signaler les états invalides.

---

## 6. Modèle conceptuel minimal

## 6.1 Variables par nœud
```text
activation_i
threshold_i
fatigue_i
memory_trace_i
excitability_i
inhibition_i
```

## 6.2 Variables par liaison
```text
conductance_ij
distance_ij
coactivity_trace_ij
plasticity_ij
decay_ij
```

## 6.3 Entrées externes
```text
stimulus(t, i, intensity)
```

## 6.4 Sorties observables
```text
active_nodes(t)
global_energy(t)
path_usage(t)
conductance_map(t)
trace_map(t)
```

---

## 7. Boucle de simulation V0

À chaque tick :

1. injection éventuelle de stimulus ;
2. calcul de l’influence reçue par chaque nœud ;
3. mise à jour de l’activation ;
4. application de la fatigue / inhibition ;
5. accumulation ou décroissance des traces locales ;
6. mise à jour des traces de co-activation ;
7. mise à jour lente des conductances ;
8. collecte des métriques ;
9. export éventuel de snapshot.

---

## 8. Paramètres configurables

La V0 doit permettre de configurer au minimum :

- taille du graphe ;
- topologie initiale ;
- distribution initiale des seuils ;
- intensité des stimuli ;
- noyau de propagation ;
- taux de dissipation ;
- intensité de fatigue ;
- intensité d’inhibition ;
- vitesse d’oubli des traces ;
- vitesse de renforcement ;
- vitesse d’affaiblissement ;
- bornes min/max des conductances ;
- durée de simulation ;
- graine aléatoire.

---

## 9. Livrables V0

## 9.1 Livrables logiciels
- moteur de simulation ;
- format de configuration ;
- format d’export des métriques ;
- outil minimal de visualisation temporelle ;
- scripts d’expériences de base.

## 9.2 Livrables scientifiques
- description des variables ;
- définition des règles de mise à jour ;
- liste des métriques ;
- protocole de test minimal ;
- cas de validation.

---

## 10. Critères d’acceptation

Le système V0 sera considéré conforme si :

1. il exécute une simulation reproductible ;
2. une activation locale se propage spatialement ;
3. cette propagation se dissipe correctement ;
4. certaines trajectoires deviennent préférentielles après répétition ;
5. les métriques permettent de constater l’émergence ou non de chemins stabilisés ;
6. le système peut être réglé dans une zone stable non triviale :
   - ni mort immédiate ;
   - ni explosion permanente.

---

## 11. Risques techniques

## 11.1 Risques majeurs
- système trop proche d’une simple diffusion sans mémoire ;
- système trop instable ;
- paramètres trop sensibles ;
- absence de vrais attracteurs ;
- renforcement local produisant uniquement des artefacts.

## 11.2 Risques de conception
- confusion entre réalisme biologique et pertinence fonctionnelle ;
- surcharge de détails prématurée ;
- manque d’instrumentation ;
- sous-définition des métriques.

---

## 12. Roadmap après V0

Si la V0 est validée, les extensions ultérieures pourront inclure :

- création et suppression de liaisons ;
- régions aux dynamiques différentes ;
- fronts anisotropes ;
- modulation régionale ;
- méta-stabilité ;
- hiérarchie de motifs ;
- proto-mémoire procédurale ;
- apprentissage multi-échelle ;
- mécanismes de consolidation hors ligne.

---

## 13. Conclusion

La V0 n’est pas une IA complète.  
C’est un **substrat expérimental** destiné à vérifier qu’un système spatio-temporel à mémoire locale peut :

- propager de l’activité ;
- conserver des traces ;
- déformer ses propres chemins ;
- et commencer à produire des structures d’apprentissage non statistiques au sens classique.

Le succès de la V0 ne se mesurera pas à sa capacité à “répondre intelligemment”, mais à sa capacité à faire émerger des **dynamiques stables, structurantes et réutilisables**.
