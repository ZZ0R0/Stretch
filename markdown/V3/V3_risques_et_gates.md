# Risques et gates — V3

## 1. Objet

Identifier les risques majeurs de la V3 et définir les gates qui conditionnent la suite de la roadmap.

---

## 2. Risques majeurs

## 2.1 Sur-ingénierie du PID
Risque :
- conserver un réseau encore trop piloté.

Mitigation :
- PID indirect uniquement ;
- indicateurs de dépendance au PID ;
- ablation PID en fin de test.

## 2.2 Inhibition trop forte
Risque :
- réseau mort ou trop fragmenté.

Mitigation :
- sweep proportion E/I ;
- sweep gain inhibiteur ;
- instrumentation par zone.

## 2.3 STDP ingérable
Risque :
- explosion paramétrique ;
- distributions de poids illisibles.

Mitigation :
- profils simples au départ ;
- bornage strict ;
- protocole A→B minimal avant complexification.

## 2.4 Fausse oscillation émergente
Risque :
- prendre un résidu de pacemaker ou de PID pour une oscillation réelle.

Mitigation :
- protocole sans pacemaker ;
- ablation pacemaker ;
- observation de phase E/I.

## 2.5 Sélectivité mémoire insuffisante
Risque :
- retour de la consolidation de masse sous une autre forme.

Mitigation :
- métriques de sparsité ;
- quotas ;
- normalisation.

---

## 3. Gate de sortie V3

Passage à V4 autorisé uniquement si :

1. inhibition inter-neuronale fonctionnelle ;
2. STDP directionnelle démontrée ;
3. PID indirect réellement indirect ;
4. au moins un régime oscillatoire émergent mesuré ;
5. sélectivité mémoire meilleure qu’en V2.

---

## 4. Si le gate échoue

- si inhibition OK mais STDP faible : itération V3.x centrée STDP ;
- si STDP OK mais oscillations absentes : itération V3.x centrée E/I ;
- si tout marche sauf performance : sprint GPU transversal avant V4.
