#let title = "Proposal"
#let subtitle = "DSA Project 5"
#let author = "Daniel King"
#let author-email = link("mailto:daking4@crimson.ua.edu")
#let class = "CS 201 Data Structures & Algorithms"
#let university = "University of Alabama"
#let term = "Fall 2026"

#let color-accent = rgb("2F3E5E")
#let color-black = rgb("000020")
#let color-offwhite = rgb("f4f4ff")
#let color-greyish = color-offwhite.mix((color-black, 30%))

#import "@preview/codly:1.3.0": *
#show: codly-init.with()


#codly(
    stroke: 1pt + color-offwhite,
    zebra-fill: color-offwhite,
    display-name: false,
    display-icon: false,
    radius: 0pt,
    inset: (x: 0.32em, y: 0.16em),
    number-align: right + top,
    number-format: it => text(fill: color-greyish)[#it],
)

#let title-block() = {
    set align(center)

    text(
        style: "normal",
        weight: 700,
        size: 18pt,
        [
            #title \
        ]
    )

    text(
        style: "normal",
        weight: 600,
        size: 16pt,
        [
            #subtitle \
        ]
    )

    text(
        style: "normal",
        weight: 500,
        size: 10pt,
        [
            \
            #class \
            #university \
            #term \
            \
            #author \<#author-email\> \
            \
        ]
    )

    // outline(title: none)
    // pagebreak()
}

#set page(
    numbering: "1",
    header: context {
        if counter(page).get().first() > 1 [
            #set text(8pt)
            *#title* #h(1fr) *#author* \
            #align(center, line(length: 100%, stroke: 0.8pt))
        ]
    },

    footer: context {
        set text(8pt)
        // align(center, line(length: 50%, stroke: 0.8pt + color-accent))
        align(center, [*#counter(page).display()*])
    }
)

#show heading: it => [
    // #set text(font: "Outfit", weight: 600)

    #if it.numbering != none [
        #counter(heading).display()
        #h(0.3em)
    ]

    #it.body\
    // #underline(it.body, stroke: 1.2pt)\
]

#show link: set text(color-accent)
#show link: underline

#show raw.where(block: false): it => {
    set text(font: "Iosevka NF", fill: color-accent)
    box(inset: (x: 2pt, y: 2pt), fill: color-offwhite)[#it]
}

#let hh(expr) = grid(
  columns: (auto, 1fr),
  gutter: 0.5em,
  expr,
  line(length: 100%, stroke: 0.7pt + black, start: (0%, 0.7%)),
)

#set text(size: 10pt)
#set image(fit: "contain", scaling: "smooth")

#let code-block(title, expr) = local(
    header: [
        #align(left)[#box(width: 1fr, inset: (bottom: 4pt), stroke: (bottom: 0.8pt), [*#title*])]
    ],
)[
    #show raw.where(block: true): it => block(
        width: 100%,
        align(center)[
            #block(width: 90%)[
                #set align(left)
                #set text(font: "Iosevka NF", size: 8pt)
                #it
            ]
        ]
    )
    #expr
]


#title-block()

// BEGIN ASSIGNMENT

#v(4em)

I'd like to write a multi-client socket program
that downloads files
from the internet over HTTP and
exposes commands
to manage the downloaded files.
Clients may connect to the program over TCP
and download files and execute file management commands
(on the system running the program).

This program would use asynchronous TCP sockets.
I'd primarily make use of the
#link("https://github.com/tokio-rs/tokio")[Tokio]
and #link("https://github.com/seanmonstar/reqwest")[reqwest] crates.
Since the former provides
#link("https://docs.rs/tokio/latest/tokio/net/struct.TcpListener.html")[`tokio::net::TcpListener`],
would it be acceptable
to use Tokio's socket abstractions
instead of SFML's
for this project?

As for the integration of data structures,
I'm likely to interact with hash maps for HTTP headers,
a priority queue for downloads,
and perhaps a B-Tree for a file database
(though this usage is likely to be contrived).
I intend to use the standard library's implementations for each of these.

// typst 0.15.1 (9dfd3a08)
// vi: set ts=4 sw=4 tw=0 :
