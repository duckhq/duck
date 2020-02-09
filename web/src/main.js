import Vue from 'vue'
import VueRouter from 'vue-router'
import App from './App.vue'
import BootstrapVue from 'bootstrap-vue'
import './styles/custom.scss'
import VueProgressBar from 'vue-progressbar'
import vueHeadful from 'vue-headful';
import Moment from 'vue-moment'
import { store } from "./store";

import { library } from '@fortawesome/fontawesome-svg-core'
import { faCheckCircle } from '@fortawesome/free-solid-svg-icons'
import { faExclamationTriangle } from '@fortawesome/free-solid-svg-icons'
import { faCodeBranch } from '@fortawesome/free-solid-svg-icons'
import { faTools } from '@fortawesome/free-solid-svg-icons'
import { faTimesCircle } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add(faCheckCircle)
library.add(faExclamationTriangle)
library.add(faCodeBranch)
library.add(faTools)
library.add(faTimesCircle)
Vue.component('font-awesome-icon', FontAwesomeIcon)

Vue.component('vue-headful', vueHeadful);

Vue.use(BootstrapVue)
Vue.use(Moment);
Vue.config.productionTip = false

Vue.use(VueProgressBar, {
  color: '#9CCC6588',
  failedColor: '#874b4b88',
  thickness: '5px',
  transition: {
    speed: '0.2s',
    opacity: '0.6s',
    termination: 300
  },
  autoRevert: false,
  location: 'top',
  inverse: false
})

Vue.use(VueRouter)
var router = new VueRouter({
  mode: 'history',
  routes: []
});

new Vue({
  router,
  store,
  render: h => h(App),
}).$mount('#app')
