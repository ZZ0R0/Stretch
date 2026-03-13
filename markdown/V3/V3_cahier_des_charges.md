# Cahier des charges — V3
## Inhibition, STDP, PID indirect

## 1. Objet

La V3 a pour objectif de résoudre les verrous majeurs identifiés après V2 :

- activité trop homogène ;
- consolidation de masse ;
- absence d’inhibition inter-neuronale ;
- plasticité corrélative non causale ;
- rôle excessif du PID ;
- oscillations imposées au lieu d’émergentes.

La V3 doit transformer le système en un substrat dont la dynamique résulte davantage des interactions internes que du seul maintien par contrôle.

---

## 2. Objectifs fonctionnels

La V3 doit permettre :

- coexistence de neurones excitateurs et inhibiteurs ;
- propagation signée ;
- plasticité dépendante du timing ;
- activité auto-entretenue avec PID indirect ;
- premières oscillations émergentes ;
- réduction de la consolidation non sélective.

---

## 3. Périmètre V3

### Inclus

- neurones excitateurs ;
- neurones inhibiteurs ;
- STDP ;
- PID indirect ;
- normalisation / sélectivité mémoire minimale ;
- protocoles de stabilité et d’évaluation ;
- benchmarks performance ciblés propagation + STDP.

### Exclus

- hiérarchie complète micro/méso/macro ;
- mémoire multi-système ;
- assemblées explicites ;
- symbolisation ;
- agent textuel ;
- refonte totale des zones.

---

## 4. Exigences fonctionnelles

## 4.1 Types neuronaux minimaux

La V3 doit au minimum supporter :

- `Excitatory`
- `Inhibitory`
- `Control`
- `Pacemaker` (maintenu comme outil d’amorçage, pas comme moteur principal)

### Contraintes
- chaque nœud a un type explicite ;
- la proportion de neurones inhibiteurs doit être configurable ;
- cible initiale recommandée : 15% à 25% d’inhibiteurs.

---

## 4.2 Propagation signée

La propagation doit dépendre du type de la source.

Exigence minimale :

- un neurone excitateur augmente l’influence reçue ;
- un neurone inhibiteur la réduit.

Le moteur doit supporter :

- gain excitateur ;
- gain inhibiteur ;
- bornes pour éviter une divergence numérique.

---

## 4.3 PID indirect

Le PID ne doit plus injecter directement une activation additive dans les nœuds de sa zone.

Le PID agit à la place sur des **paramètres de contexte**, au minimum parmi :

- seuil de zone ;
- gain de propagation de zone ;
- excitabilité de zone ;
- budget métabolique de zone.

Le système doit permettre d’activer ou de désactiver séparément chacun de ces leviers.

### Exigence clé
Le réseau doit pouvoir rester actif par propagation récurrente même lorsque l’injection directe du PID est désactivée.

---

## 4.4 STDP

Chaque arête doit pouvoir apprendre en fonction de l’ordre d’activation des nœuds pré- et post-synaptiques.

### Exigences minimales
- stockage d’un historique temporel minimal par arête ou par nœud ;
- calcul de Δt = t_post - t_pre ;
- renforcement si Δt > 0 ;
- affaiblissement si Δt < 0 ;
- bornage des conductances ;
- coexistence possible avec la plasticité V2 pendant une phase de transition.

---

## 4.5 Sélectivité mémoire minimale

La V3 doit introduire au moins un mécanisme limitant la consolidation de masse.

Mécanismes autorisés :
- normalisation hebbienne ;
- budget synaptique sortant ;
- seuil adaptatif de consolidation ;
- gating événementiel ;
- quota de connexions consolidables.

Au moins un mécanisme doit être actif dans la configuration de référence V3.

---

## 4.6 Oscillations émergentes

La V3 doit viser l’apparition d’oscillations issues de boucles E→I→E, sans dépendre exclusivement de pacemakers.

### Exigence de validation
Au moins un régime oscillatoire émergent doit être observé et mesuré avec :
- fréquence ;
- amplitude ;
- stabilité temporelle ;
- localisation spatiale.

---

## 4.7 Instrumentation obligatoire

La V3 doit exposer :

- nombre de neurones actifs par type ;
- énergie excitatrice et inhibitrice ;
- activité moyenne par zone ;
- distribution des conductances ;
- métriques STDP ;
- indice de sélectivité mémoire ;
- détection des oscillations ;
- métriques de compétition locale.

---

## 5. Exigences non fonctionnelles

- conservation du déterminisme à graine fixe ;
- compatibilité avec l’architecture modulaire existante ;
- rétro-compatibilité si possible des formats de config ;
- documentation explicite des plages de stabilité ;
- coût V3 maîtrisé pour au moins 50k nœuds en CPU ;
- préparation d’un futur sprint GPU.

---

## 6. Livrables obligatoires

- moteur V3 ;
- configurations minimales ;
- visualisation adaptée ;
- métriques exportables ;
- benchmarks propagation+STDP ;
- note d’aboutissement V3.

---

## 7. Critères d’acceptation

La V3 est validée si :

1. un réseau E/I stable fonctionne ;
2. la STDP produit un apprentissage directionnel mesurable ;
3. le PID indirect maintient des conditions-cadre sans redevenir omnipotent ;
4. une oscillation émergente est observée sans pacemaker ;
5. la consolidation de masse est réduite par rapport à V2 ;
6. aucune régression critique V2 n’est introduite.
