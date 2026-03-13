# Modélisation mathématique — V2

## 1. Activité moyenne d'une zone

Pour une zone Z :

a_zone(t) = (1 / |Z|) * Σ a_i(t)

---

# 2. Erreur de régulation

e(t) = a_target - a_zone(t)

---

# 3. Contrôleur PID

u(t) = Kp * e(t) + Ki * ∑ e(t)Δt + Kd * (e(t) - e(t-1)) / Δt

---

# 4. Injection régulée

a_i(t+1) = a_i(t) + u(t)

pour tout i appartenant à la zone.

---

# 5. Consolidation des conductances

si

w_ij > w_consolidation

pendant

T_consolidation

alors

decay = 0

---

# 6. Oscillateur pacemaker

a_i(t+1) = a_i(t) + A sin(2π f t)

---

# 7. Condition d'activité stable

a_target > θ_eff