<template>
  <div v-if="hasViews">
    <div class="ui vertical buttons fluid">
      <!-- Start button -->
      <button
        v-if="hasViews"
        class="ui button icon labeled"
        :class="{ active: currentView == null }"
        @click="visitView('/')"
      >
        <i class="right icon" :class="{ arrow: currentView == null}"></i>
        <span>{{ title }}</span>
      </button>
      <!-- View buttons -->
      <button
        v-for="view in allViews"
        :key="view.slug"
        class="ui button icon labeled"
        :class="{ active: view.slug == currentView }"
        @click="visitView(view.slug)"
      >
        <i class="right icon" :class="{ arrow: view.slug == currentView}"></i>
        <span>{{ view.name }}</span>
      </button>
    </div>
  </div>
  <div v-else>
    <!-- No views available -->
    <i>No views available</i>
  </div>
</template>

<script>
import { data } from "@/js/store.js";

export default {
  computed: {
    title() {
      return data.info.title;
    },
    currentView() {
      return data.view;
    },
    allViews() {
      if (data.info == null) {
        return null;
      }
      return data.info.views;
    },
    hasViews() {
      let foo = this.allViews;
      return foo != null && foo.length > 0;
    }
  },
  methods: {
    visitView(view) {
      if (view != undefined) {
        if (view == "/") {
          if (data.server != null && data.server !== '') {
            this.$router.push(`?server=${data.server}`);
          } else {
            this.$router.push(``);
          }
        } else {
          if (data.server != null && data.server !== '') {
            this.$router.push(`/?view=${view}&server=${data.server}`);
          } else {
            this.$router.push(`/?view=${view}`);
          }
        }
        // Send an event to the parent 
        // saying that the view was changed.
        this.$emit('view_changed')
      }
    }
  }
};
</script>