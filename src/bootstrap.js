const pages = [
  "monitor",
  "tuning",
  "fan",
  "power",
  "display",
  "keyboard",
  "settings",
];

async function loadPage(page) {
  const response = await fetch(`pages/${page}.html`);

  if (!response.ok) {
    throw new Error(`Failed to load page: ${page}`);
  }

  return await response.text();
}

async function loadPages() {
  const content = document.querySelector(".content");

  if (!content) {
    throw new Error("Main content container not found");
  }

  for (const page of pages) {
    const html = await loadPage(page);

    content.insertAdjacentHTML("beforeend", html);
  }
}

async function bootstrap() {
  try {
    await loadPages();

    await import("./js/main.js");
  } catch (error) {
    console.error("NUCtool bootstrap failed:", error);
  }
}

bootstrap();