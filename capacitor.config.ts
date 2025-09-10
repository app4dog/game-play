import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'dog.app4.game',
  appName: 'App4.Dog Game',
  webDir: 'dist/spa',
  server: {
    androidScheme: 'https'
  }
};

export default config;
