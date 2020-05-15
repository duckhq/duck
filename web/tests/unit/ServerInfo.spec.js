import Vue from 'vue'
import { shallowMount } from '@vue/test-utils'
import ServerInfo from '@/components/ServerInfo.vue'


describe('Server Info', () => {
  let moment = null
  let wrapper = null
  let data = {}

  function mount() {
    wrapper = shallowMount(ServerInfo, {
      stubs: [
        'fa-icon'
      ],
      propsData: {
        data
      }
    })
  }

  beforeEach(() => {
    moment = jest.fn().mockReturnValueOnce("a minute ago")
    Vue.filter('moment', moment)
  })

  describe("happy path", () => {
    beforeEach(() => {
      data.server = "localhost:12345"
      data.version = "1.2.3.4"
      data.info = {
        started: new Date(2020, 5, 1)
      }
      mount()
    })

    it('shows the server version info', () => {
      expect(wrapper.find("#version").text()).toBe(data.version)
    })
  
    it('shows the server host', () => {
      expect(wrapper.find("#server").text()).toBe(data.server)
    })
  
    it('shows the server started time', () => {
      expect(wrapper.find("#started").text()).toBe("a minute ago")
      expect(moment.mock.calls.length).toBe(1)
      expect(moment.mock.calls[0]).toEqual([data.info.started, "from", "now"])
    })
  })

  describe("when values are missing", () => {
    beforeEach(() => {
      data.version = null
      data.server = null
      data.info.started = null
      mount()
    })

    it('does not show version info when null', () => {
      expect(wrapper.find("#version").exists()).toBe(false)
    })
  
    it("does not show server host when null", () => {
      expect(wrapper.find("#server").exists()).toBe(false)
    })
  
    it("does not show server started time when null", () => {
      expect(wrapper.find("#started").exists()).toBe(false)
    })
  })

  describe("when data is missing", () => {
    beforeEach(() => {
      data = null
      mount()
    })

    it('does not show version info when null', () => {
      expect(wrapper.find("#version").exists()).toBe(false)
    })
  
    it("does not show server host when null", () => {
      expect(wrapper.find("#server").exists()).toBe(false)
    })
  
    it("does not show server started time when null", () => {
      expect(wrapper.find("#started").exists()).toBe(false)
    })
  })
})
