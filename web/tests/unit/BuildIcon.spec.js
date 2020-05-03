jest.mock("@/js/config", () => ({
  imageLoader: jest.fn().mockReturnValueOnce("./assets/azure.svg")
}))

import { imageLoader } from "@/js/config"
import { shallowMount } from "@vue/test-utils"
import BuildIcon from '@/components/BuildIcon.vue'

describe("BuildIcon", () => {
  let wrapper = null
  let provider = null
  
  function mount() {
    wrapper = shallowMount(BuildIcon, {
      propsData: {
        build: {
          provider
        }
      }
    })
  }
  
  it("displays svg based on build status", () => {
    provider = "azure"
    mount()
    expect(imageLoader).toHaveBeenCalledWith("./azure.svg")
    expect(wrapper.find("img").attributes().src).toBe("./assets/azure.svg")
  })
})