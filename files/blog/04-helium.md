<link href="/style/style.css" rel="stylesheet"/>
<include "header.html">

<blog-header "The most crucial accomplishment of Lev Landau", "Cyril Ulyanov", "04-06-2026">

Scientists put liquid helium into a container and cooled
it down until it was freezing cold.
Suddenly, they noticed liquid's behavior was strange: it became a superfluid with zero friction.
First of all, it somehow oozed out through microscopic holes in the glass. Even more surprisingly,
it crawled up the glass and flowed from the brim.
According to regular physics, atoms are supposed to collide, create friction, and therefore decelerate.
However, what they observed with the liquid looked like a contradiction.

<img src="/blog/assets/04-helium/container.jpg" alt="First visualization" width="18%">

Scientists tried to explain the lack of friction by looking at the liquid atom by atom, treating it like a very cold gas.
They thought that when the liquid got ultra-cold, individual helium atoms changed their properties, which resulted in what they witnessed.

Lev Landau took a completely different approach. Instead of looking at individual atoms, he studied the liquid as a whole.

When liquid helium is cooled below a specific temperature, it stops acting like a single substance. Mathematically, it splits into two completely different liquids occupying the very same space. Scientists express this using the formula:

$$
\text{ρ} = \text{ρ}_{\text{n}} + \text{ρ}_{\text{s}}
$$


The variables are respectively total density, normal fluid density, and superfluid density.

$$
\mathrm{v}_{\mathrm{c}} = \mathrm{min}_{\mathrm{p}} \left( \frac{\mathrm{E}(\mathrm{p})}{\mathrm{p}} \right)
$$

The formula calculates the speed limit $(\mathrm{v_c})$ the liquid must stay under so it doesn't have enough energy $(\mathrm{E(p)})$ and momentum $(\mathrm{p})$ to trigger friction. In other words, below this speed limit, the liquid lacks the energy to cause friction, so it moves freely.

To find this speed limit, the fluid balances two types of internal movements: phonons and rotons. Creating a phonon is expensive, because that requires enough speed to match the speed of sound. However, because helium atoms are packed tightly together, they have a shortcut: it is incredibly cheap for them to form a roton twist. Because physics always forces nature to take the path of least resistance, the liquid is guaranteed to take this cheaper shortcut the moment the energy threshold is met. When the liquid exceeds this critical velocity, it crashes into its own internal vibrations.

When Landau wrote these formulas in the 1940s, he could not test them in a lab, so he was using pure math and logic. Later, scientists finally built machines to test the fluid. The results matched Landau's math perfectly, which won him the Nobel Prize in 1962.

<include "footer.html">
