<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<svelte:head>
  <title>docx-jats — Free DOCX to JATS XML Converter</title>
  <meta name="description" content="Convert Word .docx manuscripts to JATS XML instantly. Free online tool for researchers, authors, and publishers. No sign-up required." />
  <meta name="keywords" content="JATS XML, DOCX to XML, Word to JATS, manuscript converter, journal article tag suite, academic publishing, XML converter" />
  <link rel="canonical" href="https://jats.aodihis.com" />
  <meta name="robots" content="index, follow" />

  <!-- Open Graph -->
  <meta property="og:type" content="website" />
  <meta property="og:url" content="https://jats.aodihis.com" />
  <meta property="og:title" content="docx-jats — Free DOCX to JATS XML Converter" />
  <meta property="og:description" content="Convert Word .docx manuscripts to JATS XML instantly. Free online tool for researchers, authors, and publishers. No sign-up required." />
  <meta property="og:site_name" content="docx-jats" />

  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary" />
  <meta name="twitter:title" content="docx-jats — Free DOCX to JATS XML Converter" />
  <meta name="twitter:description" content="Convert Word .docx manuscripts to JATS XML instantly. Free online tool for researchers, authors, and publishers." />

  <!-- Structured data -->
  {@html `<script type="application/ld+json">${JSON.stringify({
    "@context": "https://schema.org",
    "@type": "WebApplication",
    "name": "docx-jats",
    "url": "https://jats.aodihis.com",
    "description": "Convert Word .docx manuscripts to JATS XML instantly. Free online tool for researchers, authors, and publishers.",
    "applicationCategory": "UtilitiesApplication",
    "operatingSystem": "Any",
    "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" }
  })}</script>`}
</svelte:head>

{@render children()}
