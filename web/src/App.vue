<template>
  <div id="app" class="min-h-screen p-4">
    <!-- Page title -->
    <vue-headful :title="title" />

    <!-- Content -->
    <transition name="main" mode="out-in">
      <!-- Error -->
      <Error v-if="hasError" />

      <!-- Builds -->
      <BuildList v-else :builds="allBuilds" class="pb-4" />
    </transition>

    <!-- Settings dialog -->
    <Settings
      v-if="showViewDialog"
      :currentView="currentView"
      :views="allViews"
      :started="started"
      @close="showViewDialog = false"
    />

    <!-- Floating action button -->
    <button class="settings-button" @click="showViewDialog = true">
      <fa-icon icon="cog" fixed-width />
    </button>

    <!-- Progress -->
    <vue-progress-bar style="z-index:-1"></vue-progress-bar>
  </div>
</template>

<script>
import BuildList from "./components/BuildList.vue";
import Error from "./Error.vue";
import Settings from "./components/Settings.vue";
import { library } from "@fortawesome/fontawesome-svg-core";
import { faCog } from "@fortawesome/free-solid-svg-icons";
import { data, store } from "@/js/store.js";

library.add(faCog);

export default {
  name: "App",
  components: {
    BuildList,
    Error,
    Settings
  },
  data() {
    return {
      showViewDialog: false
    };
  },
  computed: {
    hasError() {
      return data.error;
    },
    started() {
      if (data.info == null) {
        return null;
      }
      return data.info.started;
    },
    currentView() {
      return data.view;
    },
    allBuilds() {
      if (data.builds == null) {
        return null;
      }
      return data.builds
        .slice()
        .sort((a, b) => (a.started < b.started ? 1 : -1))
        .sort((a, b) =>
          a.status != "Failed" || b.status == "Failed" ? 1 : -1
        );
    },
    allViews() {
      if (data.info == null) {
        return null;
      }
      return data.info.views;
    },
    title() {
      if (data == null || data.info == null) {
        return "Duck";
      } else {
        if (data.view != null) {
          for (var i = 0; i < data.info.views.length; i++) {
            if (data.info.views[i].slug === data.view) {
              return data.info.views[i].name;
            }
          }
        }
        return data.info.title;
      }
    }
  },
  mounting() {
    store.update(this.$Progress);
  },
  mounted() {
    setInterval(
      function() {
        // Load data for the current view.
        store.update(this.$Progress, process.env.VUE_APP_MY_DUCK_SERVER || this.$route.query.server, this.$route.query.view);
      }.bind(this),
      5000
    );
    store.update(this.$Progress, process.env.VUE_APP_MY_DUCK_SERVER || this.$route.query.server,  this.$route.query.view);
  }
};
</script>

<style scoped lang="scss">
.settings-button {
  @apply text-gray-100 border border-blue-900 bg-blue-500 text-lg py-2 px-3 rounded-full fixed right-0 bottom-0 mr-4 mb-4 transform transition duration-500 ease-in-out opacity-25 z-50
}

.settings-button:hover {
  @apply shadow-xl -translate-y-1 opacity-100
}
</style>
