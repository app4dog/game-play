#!/usr/bin/env node
/**
 * Fix for @vitejs/plugin-vue 6.0.1 TypeScript definition errors
 * 
 * The issue: Line 120 contains invalid TypeScript syntax:
 * export { ... vuePluginCjs as "module.exports", ... }
 * 
 * The fix: Remove the quotes around "module.exports" to make it valid
 */

const fs = require('fs');
const path = require('path');

function findPluginVueFiles(dir) {
  const files = [];
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory()) {
        if (entry.name.startsWith('@vitejs+plugin-vue@6.0.1')) {
          const typesFile = path.join(fullPath, 'node_modules/@vitejs/plugin-vue/dist/index.d.ts');
          if (fs.existsSync(typesFile)) {
            files.push(typesFile);
          }
        } else {
          files.push(...findPluginVueFiles(fullPath));
        }
      }
    }
  } catch (error) {
    // Ignore permission errors, etc.
  }
  
  return files;
}

function fixPluginVueTypes() {
  console.log('🔧 Fixing @vitejs/plugin-vue TypeScript definitions...');
  
  // Find all @vitejs/plugin-vue index.d.ts files in node_modules
  const nodeModulesDir = path.join(process.cwd(), 'node_modules');
  if (!fs.existsSync(nodeModulesDir)) {
    console.log('ℹ️ No node_modules directory found');
    return;
  }
  
  const pnpmDir = path.join(nodeModulesDir, '.pnpm');
  const files = fs.existsSync(pnpmDir) ? findPluginVueFiles(pnpmDir) : [];
  
  if (files.length === 0) {
    console.log('ℹ️ No @vitejs/plugin-vue 6.0.1 TypeScript files found to fix');
    return;
  }
  
  let fixedCount = 0;
  
  files.forEach(file => {
    try {
      const content = fs.readFileSync(file, 'utf8');
      
      // Fix the invalid export syntax
      const fixedContent = content.replace(
        /vuePluginCjs as "module\.exports"/g,
        'vuePluginCjs as moduleExports'
      );
      
      if (content !== fixedContent) {
        fs.writeFileSync(file, fixedContent, 'utf8');
        console.log(`✅ Fixed TypeScript definitions in: ${path.relative(process.cwd(), file)}`);
        fixedCount++;
      }
    } catch (error) {
      console.error(`❌ Failed to fix ${file}:`, error.message);
    }
  });
  
  if (fixedCount > 0) {
    console.log(`🎉 Successfully fixed ${fixedCount} @vitejs/plugin-vue TypeScript definition files`);
  } else {
    console.log('ℹ️ No TypeScript definition files needed fixing');
  }
}

// Run the fix if this script is executed directly
if (require.main === module) {
  fixPluginVueTypes();
}

module.exports = { fixPluginVueTypes };