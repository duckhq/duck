<template>
  <div v-if="hasViews">
    <ol class="leading-loose">
      <!-- Start button -->
      <li class="view-item">
        <button
          v-if="hasViews"
          :class="{ active: currentView == null }"
          @click="visitView('/')"
        >
          <span class="w-8 text-center inline-block">
            <fa-icon icon="arrow-right" v-show="currentView == null" />
          </span>
          <span class="inline-block p-2">{{ title }}</span>
        </button>
      </li>
      <!-- View buttons -->
      <li 
        class="view-item"
        v-for="view in allViews"
        :key="view.slug">
        <button
          :class="{ active: view.slug == currentView }"
          @click="visitView(view.slug)"
        >
          <span class="w-8 text-center inline-block ">
            <fa-icon icon="arrow-right" v-show="view.slug == currentView" />
          </span>
          <span class="inline-block p-2">{{ view.name }}</span>
        </button>
      </li>
    </ol>
  </div>
  <div v-else>
    <!-- No views available -->
    <span class="italic">No views available</span>
  </div>
</template>

<script>
import { library } from "@fortawesome/fontawesome-svg-core";
import { faArrowRight } from "@fortawesome/free-solid-svg-icons";

library.add(faArrowRight);

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

<style scoped lang="scss">
.view-item { 
  @apply bg-gray-400;
}

.view-item button {
  @apply font-semibold;
}

li button.active {
  @apply text-gray-100 bg-gray-600;
}

li button {
  @apply w-full text-left;
}

li:first-child {
  @apply rounded-t-lg;
}

li:last-child {
  @apply rounded-b-lg;
}
</style>