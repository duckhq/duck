<template>
  <transition name="modal">
    <div class="modal-mask flex flex-col" v-on:keyup.esc="$emit('close')" tabindex="0" ref="modal">
      <div class="w-full md:w-1/2 xl:w-1/4 min-h-full md:mx-auto my-auto">
        <div class="bg-gray-100 border border-gray-900 rounded-md">
          <div class="">
            <!-- Header -->
            <div>
              <a  class="ribbon">
                <fa-icon icon="cog" /> Settings 
              </a>
            </div>
            <!-- Body -->
            <div class="m-2">
              <!-- Tabs -->
              <div>
                <button class="inline-block p-2 cursor-pointer" :class="{ active: show_views }" @click="current='views'">
                  Views
                </button>
                <button class="inline-block p-2 cursor-pointer" :class="{ active: show_info }" @click="current='info'">
                  Information
                </button>
              </div>
              <!-- Tab content -->
              <div class="p-2 border border-gray-600 rounded-b-md h-64">
                <!-- Views -->
                <ViewList v-if="show_views" @view_changed="$emit('close')" class="" />
                <!-- Information -->
                <ServerInfo v-if="show_info" />
              </div>
            </div>
            <!-- Footer -->
            <div class="flex justify-end m-2 mt-4">
              <button class="rounded-md border border-blue-900 bg-blue-200 px-2 py-1 text-center font-semibold text-blue-900" @click="$emit('close')">Close</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script>
import ViewList from "./ViewList.vue";
import ServerInfo from "./ServerInfo.vue";
import { library } from "@fortawesome/fontawesome-svg-core";
import { faCog } from "@fortawesome/free-solid-svg-icons";

library.add(faCog);

export default {
  props: ["currentView", "views"],
  components: {
    ViewList, 
    ServerInfo
  },
  data() {
    return {
      current: 'views'
    };
  },
  computed: {
    show_views() {
      return this.current == 'views';
    },
    show_info() {
      return this.current == 'info';
    }
  },
  mounted() {
    this.$refs.modal.focus()
  }
};
</script>

<style scoped lang="scss">
/* Container */
.modal-mask {
  @apply fixed top-0 left-0 min-w-full min-h-screen transition ease-in duration-500 z-40;
  background: rgba(0, 0, 0, 0.5);
}

.modal-wrapper {
  @apply table-cell align-middle
}

.modal-container {
  @apply w-1/2 mx-auto my-0 p-4 bg-transparent transition ease-in duration-500 overflow-y-auto;
}

/* Content */
.modal-body {
  @apply p-4;
}

.active { 
  @apply border font-bold border-gray-600 rounded-t-md;
  position: relative;
  top: 1px;
  border-bottom: 1px solid white;
}

.ribbon {
  @apply my-3 pl-8 pr-3 py-1 inline-block bg-blue-700 text-white font-semibold rounded-l-sm rounded-r-md relative border-blue-800;
  left: -1.25rem;
}

.ribbon::after {
  @apply absolute left-0 bg-transparent border border-transparent w-0 h-0;
  border-width: 0 1.2em 1.2em 0;
  border-right-color: inherit;
  top: 100%;
  content: '';
}

@media screen {
  .active:focus { 
    outline: 0;
  }
  .active:active { 
    outline: none;
    border: none;
  }
}
</style>