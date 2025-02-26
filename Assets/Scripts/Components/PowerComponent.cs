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
            BatteryComponentCore battery = null,
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
            if (this.battery.PercentEnergy >= 1)
            {
                return;
            }
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
            BatteryComponent battery = null,
            ResourcesComponent resources = null,
            string burnResource = "",
            uint burnRate = 0,
            uint gainRate = 0
        ) => this.core.Instantiate(battery.core, resources.core, burnResource, burnRate, gainRate);

        public void GeneratePower() => this.core.GeneratePower();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class PowerComponentTest
    {
        [Fact]
        public void TestGeneratePowerNulls1()
        {
            PowerComponentCore power = new();
            power.Instantiate();
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
            Assert.Equal(0, battery.Energy);
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
            Assert.Equal(10, battery.Energy);
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
            Assert.Equal(0, battery.Energy);
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
            Assert.Equal(0, battery.Energy);
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
            Assert.Equal(0, battery.Energy);
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
            Assert.Equal(10, battery.Energy);
        }

        [Fact]
        public void TestOvercharge()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            power.Instantiate(battery, null, "sunlight", 0, 200);
            power.GeneratePower();
            Assert.Equal(98, battery.Energy);
        }

        [Fact]
        public void TestOverchargeDoesntConsume()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(100, 100);
            ResourcesComponentCore resources = new();
            resources.Instantiate(1, new Dictionary<string, uint> { { "coal", 1 } });
            power.Instantiate(battery, resources, "coal", 1, 200);
            power.GeneratePower();
            Assert.Equal(99, battery.Energy);
            Assert.Equal((uint)1, resources.TotalResources);
        }

        [Fact]
        public void TestChargingTo100Percent()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            power.Instantiate(battery, burnResource: "sunlight", gainRate: 1);
            power.GeneratePower();
            for (int i = 0; i < 100; i++)
            {
                power.GeneratePower();
            }
            Assert.Equal(1, battery.PercentEnergy);
            Assert.Equal("100%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestHighGain()
        {
            PowerComponentCore power = new();
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            power.Instantiate(battery, gainRate: 95);
            power.GeneratePower();
            Assert.Equal(95, battery.Energy);
            Assert.Equal(1, battery.PercentEnergy);
            Assert.Equal("100%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestTwoGeneratorsFourConsumers()
        {
            PowerComponentCore power1 = new();
            BatteryComponentCore battery1 = new();
            battery1.Instantiate();
            power1.Instantiate(battery1, gainRate: 10);

            BatteryComponentCore battery2 = new();
            PowerComponentCore power2 = new();
            battery2.Instantiate();
            power2.Instantiate(battery2, gainRate: 10);

            BatteryComponentCore battery3 = new();
            BatteryComponentCore battery4 = new();
            battery3.Instantiate();
            battery4.Instantiate();

            power1.GeneratePower();
            battery1.Balance(
                new List<BatteryComponentCore> { battery1, battery2, battery3, battery4 }
            );

            power2.GeneratePower();
            battery2.Balance(
                new List<BatteryComponentCore> { battery1, battery2, battery3, battery4 }
            );

            battery3.Balance(
                new List<BatteryComponentCore> { battery1, battery2, battery3, battery4 }
            );
            battery4.Balance(
                new List<BatteryComponentCore> { battery1, battery2, battery3, battery4 }
            );

            float totalEnergy =
                battery1.Energy + battery2.Energy + battery3.Energy + battery4.Energy;

            Assert.Equal(20, Math.Round(totalEnergy));
            Assert.Equal(5, Math.Round(battery1.Energy));
            Assert.Equal(5, Math.Round(battery2.Energy));
            Assert.Equal(5, Math.Round(battery3.Energy));
            Assert.Equal(5, Math.Round(battery4.Energy));
        }
    }
}
