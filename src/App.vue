<script setup>
const greetMsg = ref("");
const name = ref("");

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <main class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 py-10 px-4">
    <div class="max-w-4xl mx-auto">
      <h1 class="text-4xl font-bold text-center mb-8 text-gray-800 dark:text-white">
        Welcome to Tauri + Vue + Tailwind
      </h1>

      <div class="flex justify-center gap-8 mb-8">
        <a href="https://vite.dev" target="_blank" class="group">
          <img src="/vite.svg" class="h-24 w-24 transition-all duration-300 group-hover:drop-shadow-[0_0_2em_#747bff]" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank" class="group">
          <img src="/tauri.svg" class="h-24 w-24 transition-all duration-300 group-hover:drop-shadow-[0_0_2em_#24c8db]" alt="Tauri logo" />
        </a>
        <a href="https://vuejs.org/" target="_blank" class="group">
          <img src="./assets/vue.svg" class="h-24 w-24 transition-all duration-300 group-hover:drop-shadow-[0_0_2em_#249b73]" alt="Vue logo" />
        </a>
      </div>

      <p class="text-center text-gray-600 dark:text-gray-300 mb-8">
        Click on the Tauri, Vite, and Vue logos to learn more.
      </p>

      <div class="max-w-md mx-auto bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6">
        <a-form @submit.prevent="greet" layout="vertical">
          <a-form-item label="Enter your name">
            <a-input 
              id="greet-input" 
              v-model:value="name" 
              placeholder="Enter a name..."
              size="large"
            />
          </a-form-item>
          <a-form-item>
            <a-button type="primary" html-type="submit" block size="large">
              Greet
            </a-button>
          </a-form-item>
        </a-form>

        <a-alert 
          v-if="greetMsg" 
          :message="greetMsg" 
          type="success" 
          show-icon 
          class="mt-4"
        />
      </div>

      <div class="mt-8 text-center">
        <a-divider>Features</a-divider>
        <a-row :gutter="[16, 16]" class="mt-4">
          <a-col :xs="24" :sm="12" :md="8">
            <a-card hoverable>
              <template #title>
                <span class="text-blue-600">Tailwind CSS</span>
              </template>
              <p class="text-gray-600 dark:text-gray-300">
                Utility-first CSS framework for rapid UI development
              </p>
            </a-card>
          </a-col>
          <a-col :xs="24" :sm="12" :md="8">
            <a-card hoverable>
              <template #title>
                <span class="text-green-600">Auto Import</span>
              </template>
              <p class="text-gray-600 dark:text-gray-300">
                Automatic import of Vue APIs and components
              </p>
            </a-card>
          </a-col>
          <a-col :xs="24" :sm="12" :md="8">
            <a-card hoverable>
              <template #title>
                <span class="text-purple-600">Ant Design Vue</span>
              </template>
              <p class="text-gray-600 dark:text-gray-300">
                Enterprise-class UI design language and Vue components
              </p>
            </a-card>
          </a-col>
        </a-row>
      </div>
    </div>
  </main>
</template>
