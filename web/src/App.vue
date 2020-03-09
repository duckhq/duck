<template>
  <div id="app">
    <!-- Page title -->
    <vue-headful :title="title" />

    <!-- Content -->
    <transition name="main" mode="out-in">
      <!-- Error -->
      <Error v-if="hasError" />

      <!-- Builds -->
      <BuildList v-else :builds="allBuilds" class="builds" />
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
    <button class="circular blue ui icon button fab" @click="showViewDialog = true">
      <i class="icon cog"></i>
    </button>

    <!-- Progress -->
    <vue-progress-bar style="z-index:-1"></vue-progress-bar>
  </div>
</template>

<script>
import BuildList from "./components/BuildList.vue";
import Error from "./Error.vue";
import Settings from "./components/Settings.vue";
import { data, store } from "@/js/store.js";

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
        store.update(this.$Progress, this.$route.query.view);
      }.bind(this),
      5000
    );
  }
};
</script>

<style scoped>
#app {
  padding: 1rem 1rem;
  height: 100vh;
}

#app .main-enter-active,
#app .main-leave-active {
  transition: opacity 0.6s ease;
}

#app .main-enter,
#app .main-leave-to {
  opacity: 0;
}

.builds {
  padding-bottom: 1rem;
}

.fab {
  position: fixed;
  width: 50px;
  height: 50px;
  bottom: 20px;
  right: 20px;
  opacity: 0.25;
  box-shadow: 0px 0px 2px 2px rgba(0, 0, 0, 0.4) !important;
}
.fab:hover {
  opacity: 1;
  transition: 0.4s;
}
</style>