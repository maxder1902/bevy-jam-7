# quadratic falloff   
Dmg = base dmg / (1 + distance^2)   
use this for explosions   

$$
\frac{base\_dmg}{(1 + 4(dist/r)^2)} = \frac{base\_dmg}{(1 + 4\cdot dist^2/r^2)}
$$
where dist is the distance to the explosion and r is the radius of the splash damage. This formula means that at distance 0 the damage will be equal to base\_dmg, and at distance r the damage will be 20% of base dmg. If distance is bigger than r then there's no damage at all.   
   
