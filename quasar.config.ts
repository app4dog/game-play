// Configuration for your app
// https://v2.quasar.dev/quasar-cli-vite/quasar-config-file

import { defineConfig } from '#q-app/wrappers';
import { fileURLToPath } from 'node:url';
import { readFileSync, writeFileSync, existsSync, mkdirSync, copyFileSync } from 'node:fs';
import { resolve } from 'node:path';

export default defineConfig((ctx) => {
  return {
    // https://v2.quasar.dev/quasar-cli-vite/prefetch-feature
    // preFetch: true,

    // app boot file (/src/boot)
    // --> boot files are part of "main.js"
    // https://v2.quasar.dev/quasar-cli-vite/boot-files
    boot: ['i18n', 'axios'],

    // https://v2.quasar.dev/quasar-cli-vite/quasar-config-file#css
    css: ['app.scss'],

    // https://github.com/quasarframework/quasar/tree/dev/extras
    extras: [
      // 'ionicons-v4',
      // 'mdi-v7',
      // 'fontawesome-v6',
      // 'eva-icons',
      // 'themify',
      // 'line-awesome',
      // 'roboto-font-latin-ext', // this or either 'roboto-font', NEVER both!

      'roboto-font', // optional, you are not bound to it
      'material-icons', // optional, you are not bound to it
    ],

    // Full list of options: https://v2.quasar.dev/quasar-cli-vite/quasar-config-file#build
    build: {
      target: {
        browser: ['es2022', 'firefox115', 'chrome115', 'safari14'],
        node: 'node20',
      },

      typescript: {
        strict: true,
        vueShim: true,
        // extendTsConfig (tsConfig) {}
      },

      vueRouterMode: 'hash', // available values: 'hash', 'history'
      // vueRouterBase,
      // vueDevtools,
      // vueOptionsAPI: false,

      // rebuildCache: true, // rebuilds Vite/linter/etc cache on startup

      // publicPath: '/',
      // analyze: true,
      // env: {},
      // rawDefine: {}
      // ignorePublicFolder: true,
      // minify: false,
      // polyfillModulePreload: true,
      // distDir

      extendViteConf(viteConf) {
        // Ensure Vite gets explicit allowedHosts regardless of wrapper behavior
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const serverConf = ((viteConf as any).server ||= {});
        serverConf.allowedHosts = ['play.app4.dog'];

        // Configure MIME types for WASM files
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        serverConf.middlewareMode = false;
        serverConf.fs = { allow: ['..'] };
        
        // Add CORS headers for cross-origin requests
        serverConf.cors = {
          origin: ['https://play.app4.dog:9000', 'https://play.app4.dog:9001'],
          credentials: true
        };

        // Configure static file serving with proper MIME types
        if (!viteConf.define) viteConf.define = {};
        viteConf.define.__WASM_MIME_TYPE__ = '"application/wasm"';
      },
      // viteVuePluginOptions: {},

      vitePlugins: [
        [
          '@intlify/unplugin-vue-i18n/vite',
          {
            // if you want to use Vue I18n Legacy API, you need to set `compositionOnly: false`
            // compositionOnly: false,

            // if you want to use named tokens in your Vue I18n messages, such as 'Hello {name}',
            // you need to set `runtimeOnly: false`
            // runtimeOnly: false,

            ssr: ctx.modeName === 'ssr',

            // you need to set i18n resource including paths !
            include: [fileURLToPath(new URL('./src/i18n', import.meta.url))],
          },
        ],

        [
          'vite-plugin-checker',
          {
            vueTsc: true,
            eslint: {
              lintCommand: 'eslint -c ./eslint.config.js "./src*/**/*.{ts,js,mjs,cjs,vue}"',
              useFlatConfig: true,
            },
          },
          { server: false },
        ],

        // 🤓 Auto-sync WASM runtime files to public directory while keeping TypeScript types in src/
        {
          name: 'wasm-file-sync',
          buildStart() {
            // Copy WASM runtime files to public directory for serving
            const srcDir = resolve(process.cwd(), 'src/types/wasm');
            const publicDir = resolve(process.cwd(), 'public/game-engine');
            
            if (existsSync(srcDir)) {
              if (!existsSync(publicDir)) {
                mkdirSync(publicDir, { recursive: true });
              }
              
              // Copy JS and WASM files with warning headers
              const filesToSync = [
                { name: 'app4dog_game_engine.js', addWarning: true },
                { name: 'app4dog_game_engine_bg.wasm', addWarning: false }
              ];
              
              filesToSync.forEach(({ name, addWarning }) => {
                const srcFile = resolve(srcDir, name);
                const destFile = resolve(publicDir, name);
                if (existsSync(srcFile)) {
                  if (addWarning && name.endsWith('.js')) {
                    // Add warning header to JS files
                    const originalContent = readFileSync(srcFile, 'utf8');
                    const warningHeader = `/*
 * ⚠️  **DO NOT EDIT THIS FILE** ⚠️
 * This is an AUTO-GENERATED COPY from src/types/wasm/${name}
 * 
 * Source of truth: Rust code in game-engine/src/
 * Auto-generated via: wasm-pack build
 * Auto-copied via: Vite plugin in quasar.config.ts
 * 
 * Changes to this file will be OVERWRITTEN on next build!
 * To modify, edit the Rust source and run 'just build-wasm'
 */

`;
                    writeFileSync(destFile, warningHeader + originalContent);
                  } else {
                    // Copy binary files as-is
                    copyFileSync(srcFile, destFile);
                  }
                  
                  console.log(`🤓 Auto-synced WASM file: ${name}`);
                }
              });
              
              // Note: .gitignore for this directory is committed to prevent accidental edits of copies
            }
          }
        },
      ],
    },

    // Full list of options: https://v2.quasar.dev/quasar-cli-vite/quasar-config-file#devserver
    devServer: {
      https: {
        cert: './certs/play.app4.dog.pem',
        key: './certs/play.app4.dog-key.pem',
      },
      open: false, // don't open browser window automatically
      host: 'play.app4.dog',
      port: 4435,
      // Restrict dev server to approved hostnames only
      server: {
        allowedHosts: ['play.app4.dog', 'localhost'],
      },
      // CORS configuration for cross-origin requests
      cors: {
        origin: ['https://play.app4.dog:4435', 'https://play.app4.dog:8443'],
        credentials: true,
      },
      // Headers for proper MIME type handling
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Cross-Origin-Embedder-Policy': 'credentialless',
        'Cross-Origin-Opener-Policy': 'same-origin',
        'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type, Authorization'
      }
    },

    // https://v2.quasar.dev/quasar-cli-vite/quasar-config-file#framework
    framework: {
      config: {},

      // iconSet: 'material-icons', // Quasar icon set
      // lang: 'en-US', // Quasar language pack

      // For special cases outside of where the auto-import strategy can have an impact
      // (like functional components as one of the examples),
      // you can manually specify Quasar components/directives to be available everywhere:
      //
      // components: [],
      // directives: [],

      // Quasar plugins
      plugins: ['Notify'],
    },

    // animations: 'all', // --- includes all animations
    // https://v2.quasar.dev/options/animations
    animations: [],

    // https://v2.quasar.dev/quasar-cli-vite/quasar-config-file#sourcefiles
    // sourceFiles: {
    //   rootComponent: 'src/App.vue',
    //   router: 'src/router/index',
    //   store: 'src/store/index',
    //   pwaRegisterServiceWorker: 'src-pwa/register-service-worker',
    //   pwaServiceWorker: 'src-pwa/custom-service-worker',
    //   pwaManifestFile: 'src-pwa/manifest.json',
    //   electronMain: 'src-electron/electron-main',
    //   electronPreload: 'src-electron/electron-preload'
    //   bexManifestFile: 'src-bex/manifest.json
    // },

    // https://v2.quasar.dev/quasar-cli-vite/developing-ssr/configuring-ssr
    ssr: {
      prodPort: 3000, // The default port that the production server should use
      // (gets superseded if process.env.PORT is specified at runtime)

      middlewares: [
        'render', // keep this as last one
      ],

      // extendPackageJson (json) {},
      // extendSSRWebserverConf (esbuildConf) {},

      // manualStoreSerialization: true,
      // manualStoreSsrContextInjection: true,
      // manualStoreHydration: true,
      // manualPostHydrationTrigger: true,

      pwa: false,
      // pwaOfflineHtmlFilename: 'offline.html', // do NOT use index.html as name!

      // pwaExtendGenerateSWOptions (cfg) {},
      // pwaExtendInjectManifestOptions (cfg) {}
    },

    // https://v2.quasar.dev/quasar-cli-vite/developing-pwa/configuring-pwa
    pwa: {
      workboxMode: 'GenerateSW', // 'GenerateSW' or 'InjectManifest'
      // swFilename: 'sw.js',
      // manifestFilename: 'manifest.json',
      // extendManifestJson (json) {},
      // useCredentialsForManifestTag: true,
      // injectPwaMetaTags: false,
      // extendPWACustomSWConf (esbuildConf) {},
      // extendGenerateSWOptions (cfg) {},
      // extendInjectManifestOptions (cfg) {}
    },

    // Full list of options: https://v2.quasar.dev/quasar-cli-vite/developing-cordova-apps/configuring-cordova
    cordova: {
      // noIosLegacyBuildFlag: true, // uncomment only if you know what you are doing
    },

    // Full list of options: https://v2.quasar.dev/quasar-cli-vite/developing-capacitor-apps/configuring-capacitor
    capacitor: {
      hideSplashscreen: true,
    },

    // Full list of options: https://v2.quasar.dev/quasar-cli-vite/developing-electron-apps/configuring-electron
    electron: {
      // extendElectronMainConf (esbuildConf) {},
      // extendElectronPreloadConf (esbuildConf) {},

      // extendPackageJson (json) {},

      // Electron preload scripts (if any) from /src-electron, WITHOUT file extension
      preloadScripts: ['electron-preload'],

      // specify the debugging port to use for the Electron app when running in development mode
      inspectPort: 5858,

      bundler: 'packager', // 'packager' or 'builder'

      packager: {
        // https://github.com/electron-userland/electron-packager/blob/master/docs/api.md#options
        // OS X / Mac App Store
        // appBundleId: '',
        // appCategoryType: '',
        // osxSign: '',
        // protocol: 'myapp://path',
        // Windows only
        // win32metadata: { ... }
      },

      builder: {
        // https://www.electron.build/configuration/configuration

        appId: 'dog.app4.game',
      },
    },

    // Full list of options: https://v2.quasar.dev/quasar-cli-vite/developing-browser-extensions/configuring-bex
    bex: {
      // extendBexScriptsConf (esbuildConf) {},
      // extendBexManifestJson (json) {},

      /**
       * The list of extra scripts (js/ts) not in your bex manifest that you want to
       * compile and use in your browser extension. Maybe dynamic use them?
       *
       * Each entry in the list should be a relative filename to /src-bex/
       *
       * @example [ 'my-script.ts', 'sub-folder/my-other-script.js' ]
       */
      extraScripts: [],
    },
  };
});
