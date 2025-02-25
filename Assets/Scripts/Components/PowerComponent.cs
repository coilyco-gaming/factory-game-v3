namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;

    public class PowerComponentCore
    {
        private string burnResource = ""; // ex: coal
        private uint burnRate = 0; // ex: burn 1 coal per tick
        private uint gainRate = 0; // ex: gain 10 energy per tick

        private BatteryComponentCore battery = new();
        private ResourcesComponentCore resources = new();

        public void Instantiate(
            BatteryComponentCore battery,
            ResourcesComponentCore resources = null,
            string burnResource = "",
            uint burnRate = 0,
            uint gainRate = 0
        )
        {
            this.battery = battery ?? new BatteryComponentCore();
            this.resources = resources ?? new ResourcesComponentCore();
            this.burnResource = burnResource;
            this.burnRate = burnRate;
            this.gainRate = gainRate;
        }

        public void GeneratePower()
        {
            uint resourcesToBurn = this.resources.Resources.GetValueOrDefault(
                this.burnResource,
                (uint)0
            );
            if (resourcesToBurn >= this.burnRate)
            {
                this.resources.ConsumeResources(this.burnResource, this.burnRate);
                this.battery.Energy += this.gainRate;
            }
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class PowerComponent : MonoBehaviour
    {
        protected readonly PowerComponentCore core = new();

        public void Instantiate(
            BatteryComponent battery,
            ResourcesComponent resources,
            string burnResource,
            uint burnRate = 0,
            uint gainRate = 0
        ) => this.core.Instantiate(battery.core, resources.core, burnResource, burnRate, gainRate);

        public void GeneratePower() => this.core.GeneratePower();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class PowerComponentTest
    {
        [Fact]
        public void TestGeneratePowerNulls1()
        {
            PowerComponentCore power = new();
            power.Instantiate(null, null, "", 0, 0);
            power.GeneratePower();
        }

        [Fact]
        public void TestGeneratePowerNulls2()
        {
            PowerComponentCore power = new();
            ResourcesComponentCore resources = new();
            resources.Instantiate(1, new Dictionary<string, uint> { { "coal", 1 } });
            power.Instantiate(null, resources, "coal", 0, 0);
            power.GeneratePower();
        }

        [Fact]
        public void TestGeneratePowerZeroes()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            ResourcesComponentCore resources = new();
            power.Instantiate(battery, resources, "", 0, 0);
            power.GeneratePower();
            Assert.Equal((uint)0, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestGeneratePower()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            ResourcesComponentCore resources = new();
            resources.Instantiate(1, new Dictionary<string, uint> { { "coal", 1 } });
            power.Instantiate(battery, resources, "coal", 1, 10);
            power.GeneratePower();
            Assert.Equal((uint)10, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestConsumeResourcesNoneAvailable()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            ResourcesComponentCore resources = new();
            resources.Instantiate(1, new Dictionary<string, uint> { { "coal", 0 } });
            power.Instantiate(battery, resources, "coal", 1, 10);
            power.GeneratePower();
            Assert.Equal((uint)0, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestConsumeResourcesDifferentAvailable()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            ResourcesComponentCore resources = new();
            resources.Instantiate(1, new Dictionary<string, uint> { { "wood", 0 } });
            power.Instantiate(battery, resources, "coal", 1, 10);
            power.GeneratePower();
            Assert.Equal((uint)0, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestConsumeMoreThanAvailable()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            ResourcesComponentCore resources = new();
            resources.Instantiate(1, new Dictionary<string, uint> { { "coal", 1 } });
            power.Instantiate(battery, resources, "coal", 2, 10);
            power.GeneratePower();
            Assert.Equal((uint)0, battery.Energy);
            Assert.Equal((uint)1, resources.TotalResources);
        }

        [Fact]
        public void TestSolarPower()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            power.Instantiate(battery, null, "sunlight", 0, 10);
            power.GeneratePower();
            Assert.Equal((uint)10, battery.Energy);
        }

        [Fact]
        public void TestOvercharge()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(100, 100);
            power.Instantiate(battery, null, "sunlight", 0, 200);
            power.GeneratePower();
            Assert.Equal((uint)100, battery.Energy);
        }
    }
}
