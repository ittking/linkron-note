<script setup>
import { theme } from "ant-design-vue"

const themeConfig = {
  token: {
    colorPrimary: '#020617',
  },
  algorithm: theme.compactAlgorithm,
}

const greetMsg = ref("");
const name = ref("");

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <a-config-provider :theme="themeConfig">
    <main class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 py-6 px-3">
      <div class="max-w-md mx-auto bg-white dark:bg-gray-800 rounded-lg shadow-xl p-4">
        <a-form @submit.prevent="greet" layout="vertical">
          <a-form-item label="Enter your name">
            <a-input 
              id="greet-input" 
              v-model:value="name" 
              placeholder="Enter a name..."
            />
          </a-form-item>
          <a-form-item>
            <a-button type="primary" html-type="submit" block>
              Greet
            </a-button>
          </a-form-item>
        </a-form>

        <a-alert 
          v-if="greetMsg" 
          :message="greetMsg" 
          type="success" 
          show-icon 
          class="mt-3"
        />
      </div>
    </main>
  </a-config-provider>
</template>
