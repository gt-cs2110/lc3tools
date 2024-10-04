import Editor from "../components/editor/Editor.vue";
import Simulator from "../components/simulator/Simulator.vue";
import { createWebHistory, createRouter } from "vue-router";

export default createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/editor',
      name: 'editor',
      component: Editor
    },
    {
      path: '/simulator',
      name: 'simulator',
      component: Simulator
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/editor'
    }
  ]
})
