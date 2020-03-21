import Vue from 'vue'
import Moment from 'vue-moment'
import VueRouter from 'vue-router'
import vueHeadful from 'vue-headful';
import VueProgressBar from 'vue-progressbar'

import App from './App.vue'
import { data } from "@/js/store.js";

import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import "@/assets/styles/main.css";

Vue.component('vue-headful', vueHeadful);
Vue.component('fa-icon', FontAwesomeIcon)

Vue.use(Moment);
Vue.use(VueRouter);

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

new Vue({
    data,
    router: new VueRouter({
        mode: 'history',
        routes: []
    }),
    render: h => h(App)
}).$mount('#app')