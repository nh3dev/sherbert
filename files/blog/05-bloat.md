[article]
title = "Size does matter, actually"
author = "Nick Borș"
date = "27-06-2026"
---
![Graph of page weight over time](/blog/assets/05-bloat/bloat_graph.jpg)

I'd like to start with this graph I made[^1].

What is it that you notice?

> ...an upwards trend?

And do you think that it is justified? Has web content really gotten
130x better than it was just 30 years ago to warrant the same
increase from ~200KB to a median of ~2.5MB? I think not.

Images, videos, and text, the _real_ content, has gotten both more
accessible and more prevalant -- a good thing make no mistake --
but I find myself feeling disenchanted with the current state of
the Internet. Where are our good ol' 88x31px banners? Where is the
charm that once was, and where is the craftsmanship that made small
efficient websites commonplace?

Today I crawl about the web, encumbered by A.I-generated _slop_,
ads (and A.I-generated ads), upheld by megabytes of unreadable
minified javascript, veiled in sleek modern ui sanitized of any
character, meaning, or memorability. A wolf in sheeps clothing.
Though, thats not to say that a subculture of performance-oriented,
likely similarly frustrated people dont exist. And its on you whom
I call upon to fix this maddness.

Static web-pages built upon HTML and CSS offer a simple, streamlined
and secure way to interact with the world, share ideas, write
articles, research you name it. Even the entrepeneur-developers of
the world can learn to thrive through doing things the _old way_.
I think "static" is a bit of a misnomer -- you can get a lot done
with just a simple http server, a pinch of CSS, and some grit.
Notice the lack of javascript -- its not as necessary as you think.
Web frameworks, optimising for the _developer experience_ have lost
my trust. The debt created by their speed and ergonomics is paid
for in full by the user in the form of huge payloads. Analytics,
tracking, and pop-ups serve to only get in your way, and, upon
pealing back the rancid and rampant overgrowth, you will see the
reliable brick and mortar of the internet: HTML and CSS.  That is 
where the _real_ content lies. Note, this isnt to say that javascript
is necessarily bad, but I think that it is a breeding ground to an
attitude towards software which disrespects the user.  The same can
be said for programs outside of the web -- as electron apps choose
to waste your time, memory, and space for ease of development and
promises of cross-compatibility. They do not respect you -- or so
it seems to me at least -- they treat their users as dispensable.
If you are to take part in the ever-growing Internet, be wary of
following such design trends, and respect your customers and
peers alike.

There is this notion floating about that us _hackers_[^2] only care
from our own technical point of views, and carry radical philosophy
which only makes sense within our social circles. It is not so. The
mindset of optimising and unencumbering your users extends to even
main-stream, large, transnational corporations, it's rare, but it
happens. The impacts of speed, whilst not explicitly clear to the
layman, translates into annoyance felt by us all.  On the contrary,
good leadership and a talented team of developers at
[MacMaster-Carr](https://www.mcmaster.com/) helped make the minimalist
site a staple of modern construction, whilst being about as main-stream
as it gets.  Open their web-page. I encourage you. Take a look at
how fast it loads, and remember this when you are sold the lied to
that "websites are just more complex nowadays, so of course their
bigger".  This catalogue contains _thousands_ of unique products,
yet it not only loads quick thanks to clever engineering, but is
actually _a joy to use_. The user experience at MacMaster-Carr is a
dream of both engineers and UI designers. It is not cluttered. It is
not difficult to use like other commercial catalogues such as eBay
or Amazon with their endless nested drop-downs and inept search
functions (if you have used eBay, I'm sure you know what i mean).

> But how could such a minimalist, function-over-form attitude ever
promulgate in the corporate world? Don't shareholders chase sleek,
modern design and dont product managers demand they look better
than their competitors?

Yes, they typically do dont they? I think thats bad leadership
bandwagoning on the promise of dividends for investors through the
use of modern bloat painted in the lead paint that is modern ui
trends. The truth is that this mentality is in a false dichotomy
with the demands of the business world. In reality, users will come
back time and time again when something is boring and "just works".
If you are in a leadership position, try to see this case study as
an opportunity to be inspired. If you are a regular non-technical
user, admire the ordinary and stop and smell the roses. And lastly,
and most likely, if you are a developer, be inspired by the masterful
showcase of techniques which dance backstage behind the boring old
website:

- Image atlases to reduce network requests. They just send one strip
- Images with fixed widths -- no expensive DOM redraws. This website
skips every re-layout it can.
- Inlining critical css to prevent layout shifts.
- Pages are pre-loaded when you hover over links so theyre ready
for when you click. The use of assumptions and heuristics to improve
UX at the expense of _their_ server workload sounds to me like the
antithesis of Big. Bloat. Notice how they respect your time, and
value it over their computational resources?
- Client-side caching via service workers -- they (literally) stop
you from making unecessary requests.
- Javascript bundling. Though they admittedly use a lot, they are
careful enough to only include what is used on a per-page basis.

...and many more I probably havnt yet discovered.

My point being, it is not only _viable_ but _beneficial_[^3] to be
speedy, and understandable (with studies reporting 8% and 10%
increase in conversion rates, and order value growing by 9.2% and
1.9% from a mere 0.1s difference in load times for retail and travel
sites respectively[^4]).  Thats what retains users -- not
pestering them with useless AI chatbots and banners that seem to
always block your clicks, returning time and time again after your
dismissal.

As for individuals (hopefully such as yourself) allied to the fight
against bloat, there too exists a space carved out on the internet.
I'm not refering to your popular chat platforms, such as Discord,
Telegram, and X (formerly Twitter), of course. They are subject to the
whims of politics, legislation, bankrupcy and serve only to sell
your (meta) data away to the highest bidder -- literally. No, I
mean webrings, IRC, XMPP, mailing lists and other little nerdy and
geeky communities (such as this one!). Mass media makes you the
product. Strive to dominate the surrounding technologies, own them if 
you can, lest they dominate you. Governments are getting more and
more comfortable in takeing away your liberties (at least in some
parts of the world) and whether or not you believe that, or whether
it applies to where you live, it should still be important to you
to keep, practice, and maintain your rights. This is yet another
way in which the modern web disrespects you, the individual. So if
you needed a sign to join these little unknown communities hidden
from the public eye, maybe this is it (consider
[nh3.dev](https://nh3.dev)?).

Anyways, enough rambling.

## TL;DR

<summary>

Make websites the old way -- regardless of if you are an
individual or not -- and respect users time, privacy, and resources.
You can check your performance and page weight at [Cloudflare's
scanning service](https://radar.cloudflare.com/scan/) under the
_Network_ tab to see where you stand currently.

</summary>

[^1]: Data accsessed on 27th Jul 2026, kindly provided by the
[Http Archaive](https://httparchive.org/) (on a lean ~447KB
[page](https://httparchive.org/reports/page-weight), which I highly
recommend checking out, which just so happens to follow the principles
outlined here).  Graph was made with R's
[tidyverse](https://tidyverse.org/) meta-package.

[^2]: As defined in RFC-1983 DOI: [10.1787/RFC1983](https://doi.org/10.17487/RFC1983).

[^3]: This claim is corroborated by Web.dev's 
[_Why speed matters_](https://web.dev/learn/performance/why-speed-matters) 
and Ikášová, Tereza & Klepek, Martin. (2024). _The impact of website
performance on business sales_. Financial Internet Quarterly. 20.
81-91. DOI: [10.2478/fiqf-2024-0007](https://doi.org/10.2478/fiqf-2024-0007)
and others you can easily find. This is just what I read.

[^4]: Deloitte's _Milliseconds make Millions_
[PDF](https://www.deloitte.com/content/dam/assets-zone2/ie/en/docs/services/consulting/2023/Milliseconds_Make_Millions_report.pdf)

<br>

### Cool further reading/practical advice
- Page weight based clubs (self explanatory):
  - [250KB.club](https://250kb.club)
  - [512KB.club](https://512kb.club)
  - [1MB.club](https://1mb.club)
- The [No-JS club](https://no-js.club/) (self explanatory).
- [Danluu's post on web-bloat](https://danluu.com/web-bloat/) is
about web-bloat focusing on connection speeds.
- This triad. You can open, and steal their ideas/css/attitude as you see fit:
  - [This is a motherfucking website.](http://motherfuckingwebsite.com/)
  - [This is a better motherfucking website.](http://bettermotherfuckingwebsite.com/)
  - [This is a best motherfucking website.](http://bestmotherfuckingwebsite.com/)
- [Contrast Rebellion](https://contrastrebellion.com/)'s article
about contrast which embodies a wider premise of function over form.
- This site dedicated to [System font-stacks](https://systemfontstack.com/); 
they should be prefered almost always (imo).
