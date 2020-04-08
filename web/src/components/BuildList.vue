<template>
  <div :key="skeleton" v-if="loading" class="grid gap-4 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
    <div v-for="build in getBuilds" :key="build.id" >
      <SkeletonBuild class="skeleton" /> 
    </div>
  </div>
  <div v-else :key="real">
    <transition-group name="build" tag="div" class="grid gap-4 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
    <div v-for="build in getBuilds" :key="build.id">
        <Build :build="build" />
    </div>
  </transition-group>
  </div>
</template>

<script>
import Build from "./Build.vue";
import SkeletonBuild from "./SkeletonBuild.vue";

export default {
  props: {
    builds: {
      type: Array
    }
  },  
  components: {
    Build,
    SkeletonBuild
  },
  methods: {
    skeletonBuilds: function() {
      return [...Array(6).keys()].map(id => ({
        id,
        isSkeleton: true
      }));
    }
  },
  computed: {
    loading: function() {
      return this.builds == null;
    },
    getBuilds: function() {
      const hasRealBuilds = this.builds !== null && this.builds.length > 0;
      if(hasRealBuilds)
        return this.builds;
      else 
        return this.skeletonBuilds();
    }
  }
};
</script>

<style scoped>
.build-enter-active,
.build-leave-active {
  transition: 1s cubic-bezier(0.59, 0.12, 0.34, 0.95);
  transition-property: opacity, transform;
}
.build-move {
  transition: transform 1s;
  transition-property: opacity, transform;
}
.build-enter {
  opacity: 0;
  transform: translateX(125px) scaleY(1) scaleX(1);
}
.build-enter-to {
  opacity: 1;
  transform: translateX(0) scaleY(1);
}
.build-leave-active {
  position: absolute;
}
.build-leave-to {
  opacity: 0;
  transform: translateX(0);
  transform-origin: center top;
}
</style>