<template>
  <transition name="modal">
    <div class="modal-mask flex flex-col" v-on:keyup.esc="$emit('close')" tabindex="0" ref="modal">
      <div class="w-full md:w-1/2 xl:w-1/4 min-h-full md:mx-auto my-auto">
        <div class="bg-gray-100 border border-gray-900 rounded-md p-2">
          <div class="">
            <!-- Header -->
            <div class="my-3">
              <a>
                <fa-icon icon="cog" /> Settings 
              </a>
            </div>
            <!-- Body -->
            <div class="">
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
            <div class="flex justify-end mt-4">
              <button class="rounded-md border border-blue-900 bg-blue-200 p-2 text-center font-semibold text-blue-900" @click="$emit('close')">Close</button>
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