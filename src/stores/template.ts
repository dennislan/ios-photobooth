import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface TemplateConfig {
  layout: 'grid' | 'free' | 'single';
  padding: number;
  bgColor: string;
}

export interface Template {
  id: string;
  name: string;
  config: TemplateConfig;
  preview?: string;
}

export const useTemplateStore = defineStore('template', () => {
  const templateId = ref<string>('');
  const templates = ref<Template[]>([]);
  const selectedTemplate = ref<Template | null>(null);

  function setTemplateId(id: string) {
    templateId.value = id;
    selectedTemplate.value = templates.value.find(t => t.id === id) || null;
  }

  function addTemplate(template: Template) {
    templates.value.push(template);
  }

  function removeTemplate(id: string) {
    templates.value = templates.value.filter(t => t.id !== id);
    if (templateId.value === id) {
      templateId.value = '';
      selectedTemplate.value = null;
    }
  }

  return {
    templateId, templates, selectedTemplate,
    setTemplateId, addTemplate, removeTemplate,
  };
});
