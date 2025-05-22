import fs from 'fs/promises';
import { marked } from 'marked';
import sanitizeHtml from 'sanitize-html';

async function convert2markdown(x: string): Promise<string> {
  x = x.replace(/- \[x\]/gi, '±§±§±§±');
  x = x.replace(/- \[ \]/g, '±§±§±§§±');

  let content = await marked.parse(x); // safer than `marked(x)`

  content = String(sanitizeHtml(content, {
    allowedTags: sanitizeHtml.defaults.allowedTags,
    allowedAttributes: {
      ...sanitizeHtml.defaults.allowedAttributes,
      a: [],
      code: ['class'],
      img: ['src', 'srcset', 'alt', 'title', 'width', 'height', 'loading'],
    },
    allowedSchemes: sanitizeHtml.defaults.allowedSchemes,
  }));

  content = content.replace(
    /±§±§±§§±/g,
    `<br/><input type='checkbox' class='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600' disabled>`
  );

  content = content.replace(
    /±§±§±§±/g,
    `<br/><input type='checkbox' class='w-4 h-4 text-green-500 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600' checked disabled>`
  );

  content = content.replace(
    /\[!IMPORTANT\]/g,
    `<span class="bg-green-100 text-green-800 text-xs font-medium me-2 px-2.5 py-0.5 rounded dark:bg-green-900 dark:text-green-300">IMPORTANT</span>`
  );
  content = content.replace(
    /\[!NOTE\]/g,
    `<span class="bg-blue-100 text-blue-800 text-xs font-medium me-2 px-2.5 py-0.5 rounded dark:bg-blue-900 dark:text-blue-300">NOTE</span>`
  );
  content = content.replace(
    /\[!WARNING\]/g,
    `<span class="bg-yellow-100 text-yellow-800 text-xs font-medium me-2 px-2.5 py-0.5 rounded dark:bg-yellow-900 dark:text-yellow-300">WARNING</span>`
  );
  content = content.replace(
    /\[!CAUTION\]/g,
    `<span class="bg-red-100 text-red-800 text-xs font-medium me-2 px-2.5 py-0.5 rounded dark:bg-red-900 dark:text-red-300">CAUTION</span>`
  );

  return content;
}


const INPUT_FILE = './database/packages.json';
const INPUT_FILE2 = './database/programs.json';

async function purify(fileName: string) {
  const data = await fs.readFile(fileName, 'utf-8');
  const repos = JSON.parse(data);

  for (const repo of repos) {
    if (repo.readme_content) {
      repo.readme_content = await convert2markdown(repo.readme_content);
    }
  }

  await fs.writeFile(fileName, JSON.stringify(repos), 'utf-8');
}

async function main() {
  await purify(INPUT_FILE);
  await purify(INPUT_FILE2);
}

main().catch(console.error);
