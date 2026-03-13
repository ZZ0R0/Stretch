# Cahier des charges — V4
## Dopamine, reward, éligibilité, entrées/sorties minimales

## 1. Objet

La V4 a pour objectif d’introduire l’apprentissage guidé dans Stretch.

Après la V3, le système dispose :
- d’une dynamique interne crédible ;
- d’inhibition inter-neuronale ;
- d’une STDP directionnelle ;
- d’un PID indirect ;
- d’une sélectivité synaptique partielle.

Mais il reste incapable de :
- distinguer un bon comportement d’un mauvais ;
- apprendre sur signal de récompense ;
- recevoir des entrées structurées ;
- produire des sorties interprétables.

La V4 doit lever ces verrous.

---

## 2. Objectifs fonctionnels

La V4 doit permettre :

- l’injection d’un reward scalaire externe ;
- la propagation ou diffusion d’un signal dopaminergique minimal ;
- l’accumulation de traces d’éligibilité par arête ;
- une plasticité modulée par reward × eligibility ;
- des entrées structurées minimales ;
- des sorties lisibles minimales ;
- des tâches élémentaires de décision supervisée par reward.

---

## 3. Périmètre V4

### Inclus
- reward externe simple ;
- dopamine tonique et phasique minimale ;
- traces d’éligibilité ;
- modulation de la plasticité ;
- asymétrie STDP si nécessaire ;
- zone(s) d’entrée ;
- zone(s) de sortie ;
- readout simple ;
- protocoles d’apprentissage guidé.

### Exclus
- prédiction de récompense TD complète ;
- récompense intrinsèque ;
- hiérarchie complète des zones ;
- mémoire de travail ;
- assemblées explicites ;
- interface textuelle ;
- symbolisation.

---

## 4. Exigences fonctionnelles

## 4.1 Reward externe

Le système doit accepter un signal :

```text
r(t) ∈ [-1, +1]
```

Types minimaux :
- reward positif ;
- reward nul ;
- reward négatif.

Ce signal doit pouvoir être :
- global ;
- ou injecté dans une ou plusieurs zones cibles.

---

## 4.2 Canal dopaminergique minimal

Le système doit supporter un ou plusieurs signaux dopaminergiques :

- niveau tonique ;
- burst phasique ;
- dip phasique.

Contraintes :
- bornage ;
- latence configurable ;
- découplage d’avec le PID indirect ;
- possibilité d’activation/désactivation en config.

Le signal dopaminergique ne doit pas agir directement sur l’activation brute des neurones standards.  
Il doit moduler :
- la plasticité ;
- la consolidation ;
- éventuellement la fenêtre d’apprentissage.

---

## 4.3 Traces d’éligibilité

Chaque arête doit posséder une trace d’éligibilité `e_ij`.

Exigences minimales :
- accumulation locale suite à l’activité récente ;
- décroissance exponentielle ;
- couplage avec STDP ou événements synaptiques ;
- bornage.

La V4 doit pouvoir apprendre avec reward retardé de quelques ticks au moins.

---

## 4.4 Plasticité modulée

La mise à jour effective d’un poids doit dépendre :
- de la STDP locale ;
- de la trace d’éligibilité ;
- du reward ou signal dopaminergique.

Le moteur doit permettre de comparer :
- STDP seule ;
- STDP + dopamine ;
- STDP + eligibility + reward.

---

## 4.5 Entrées minimales

La V4 doit disposer d’une vraie interface d’entrée, distincte d’un simple stimulus arbitraire.

Exigences minimales :
- sous-graphe d’entrée dédié ;
- mapping pattern externe → activation d’entrée ;
- plusieurs classes de patterns ;
- reproductibilité à graine fixe.

Exemples admissibles :
- 2 à 4 catégories ;
- patterns binaires simples ;
- micro-séquences discrètes.

---

## 4.6 Sorties minimales

La V4 doit disposer d’une vraie interface de sortie.

Exigences minimales :
- sous-graphe de sortie dédié ;
- lecture d’une décision simple ;
- comparaison à une cible ;
- calcul d’accuracy ou reward.

Exemples :
- choix binaire ;
- 3 classes ;
- comparaison winner-take-all entre groupes de sortie.

---

## 4.7 STDP asymétrique

La V4 doit permettre de quitter le profil V3 symétrique.

Exigence minimale :
- `A_plus != A_minus`
- et/ou `tau_plus != tau_minus`

Le profil asymétrique doit être configurable et benchmarkable.

---

## 4.8 Gating de consolidation

La consolidation ne doit plus dépendre uniquement de l’activité de fond.

Au moins un mécanisme doit être actif :
- gating par reward ;
- gating par dépassement d’un seuil événementiel ;
- seuil de consolidation adaptatif.

---

## 4.9 Instrumentation obligatoire

La V4 doit exposer :

- reward courant ;
- dopamine courante ;
- distribution des traces d’éligibilité ;
- poids avant/après reward ;
- activité des zones d’entrée ;
- activité des zones de sortie ;
- accuracy / reward cumulé ;
- comparaison STDP vs STDP+reward ;
- métriques de consolidation.

---

## 5. Exigences non fonctionnelles

- déterminisme à seed fixe ;
- configuration totalement externe ;
- comparabilité des runs V3/V4 ;
- possibilité de désactiver individuellement dopamine, reward, eligibility, I/O ;
- maintien d’une échelle de 50k à 500k nœuds pour les modes non viz lourds ;
- préparation d’un futur sprint GPU.

---

## 6. Livrables obligatoires

- moteur V4 ;
- configs minimales ;
- protocoles d’apprentissage guidé ;
- benchmarks ;
- visualisation adaptée ;
- note d’aboutissement V4.

---

## 7. Critères d’acceptation

La V4 est validée si :

1. reward influence réellement les poids appris ;
2. les traces d’éligibilité permettent une association reward retardé ↔ activité récente ;
3. les patterns d’entrée distincts produisent des états distincts ;
4. la sortie obtient un résultat au-dessus du hasard sur une tâche simple ;
5. la dopamine reste un modulateur et non un second contrôleur caché ;
6. la stabilité globale du système est conservée.
