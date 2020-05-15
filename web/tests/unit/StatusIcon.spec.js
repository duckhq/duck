import { shallowMount } from "@vue/test-utils"
import StatusIcon from "@/components/StatusIcon"

describe("StatusIcon", () => {
  let status = null;
  let wrapper = null;

  const expectations = {
    Success: "check-circle",
    Failed: "exclamation-triangle",
    Running: "running",
    Canceled: "stop-circle",
    Queued: "clock",
    Skipped: "stop-circle"
  }

  function mount() {
    wrapper = shallowMount(StatusIcon, {
      stubs: [
        "fa-icon"
      ],
      propsData: {
        build: {
          status
        }
      }
    })
  }

  Object.keys(expectations).forEach(buildStatus => {
    
    it(`should show '${expectations[buildStatus]}' for '${buildStatus}' build status`, () => {
      status = buildStatus
      mount()
      const faIcon = wrapper.find('fa-icon-stub')
      expect(faIcon.attributes().icon).toBe(expectations[buildStatus])
    })

  })

  it("should gracefully handle null build status", () => {
    status = null
    mount()
    const faIcon = wrapper.find('fa-icon-stub')
    expect(faIcon.attributes().icon).toBe("question-circle")
  })
})