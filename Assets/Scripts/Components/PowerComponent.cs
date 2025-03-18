namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using Assets.Scripts.Core;
    using UnityEngine;

    [Serializable]
    public class PowerComponentCore
    {
        private string burnResource = ""; // ex: coal
        private uint burnRate = 0; // ex: burn 1 coal per tick
        private uint gainRate = 0; // ex: gain 10 energy per tick

        private BatteryComponentCore battery;
        private ResourcesComponentCore resources;
        private WorldObjectCore worldObject;

        public PowerComponentCore(
            WorldObjectCore worldObject,
            BatteryComponentCore battery,
            ResourcesComponentCore resources,
            string burnResource = "",
            uint burnRate = 0,
            uint gainRate = 0
        )
        {
            this.worldObject = worldObject;
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Power component requires a battery component"
                );
            this.resources =
                resources
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Power component requires a resources component"
                );
            this.burnResource = burnResource;
            this.burnRate = burnRate;
            this.gainRate = gainRate;
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", this.worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);

            if (this.battery.PercentEnergy >= 1)
            {
                return new();
            }
            uint resourcesToBurn = this.resources.resources.GetValueOrDefault(
                this.burnResource,
                (uint)0
            );
            if (resourcesToBurn >= this.burnRate)
            {
                // Consume no resources if the battery is full
                try
                {
                    this.battery.Energy += this.gainRate;
                    this.resources.ConsumeResources(this.burnResource, this.burnRate);
                }
                catch (BatteryComponentCore.BatteryCapacityException) { }
            }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;

    public class PowerComponentTest
    {
        private WorldObjectCore WorldObject(
            GameControllerCore gameController,
            uint startingEnergy,
            uint capacity,
            uint gainRate = 0
        )
        {
            BatteryComponentCore battery = new(new WorldObjectCore(null), startingEnergy, capacity);
            ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
            WorldObjectCore core = new(null)
            {
                battery = battery,
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            if (gainRate != 0)
            {
                core.power = new PowerComponentCore(
                    new WorldObjectCore(null),
                    battery,
                    resources,
                    gainRate: gainRate
                );
            }
            core.guid = core.CreateGuid();
            gameController.worldObjects ??= new();
            if (!gameController.worldObjects.ContainsKey(core.GridPosition))
            {
                gameController.worldObjects[core.GridPosition] = new();
            }
            gameController.worldObjects[core.GridPosition][core.guid] = core;
            return core;
        }

        [Fact]
        public void TestGeneratePowerZeroes()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null));
            ResourcesComponentCore resources = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            PowerComponentCore power = new(new WorldObjectCore(null), battery, resources, "", 0, 0);
            power.Tick(gameController);
            Assert.Equal(0, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestGeneratePower()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 0, 100);
            ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
            resources.CreateResources("coal", 1);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                resources,
                "coal",
                1,
                10
            );
            power.Tick(gameController);
            Assert.Equal(10, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestConsumeResourcesNoneAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null));
            ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                resources,
                "coal",
                1,
                10
            );
            power.Tick(gameController);
            Assert.Equal(0, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestConsumeResourcesDifferentAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null));
            ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                resources,
                "coal",
                1,
                10
            );
            power.Tick(gameController);
            Assert.Equal(0, battery.Energy);
            Assert.Equal((uint)0, resources.TotalResources);
        }

        [Fact]
        public void TestConsumeMoreThanAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null));
            ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
            resources.CreateResources("coal", 1);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                resources,
                "coal",
                2,
                10
            );
            power.Tick(gameController);
            Assert.Equal(0, battery.Energy);
            Assert.Equal((uint)1, resources.TotalResources);
        }

        [Fact]
        public void TestSolarPower()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 0, 100);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                new ResourcesComponentCore(new TestResourcesGameContent(), 1, 1),
                "sunlight",
                0,
                10
            );
            power.Tick(gameController);
            Assert.Equal(10, battery.Energy);
        }

        [Fact]
        public void TestOvercharge()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 0, 100);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                new ResourcesComponentCore(new TestResourcesGameContent(), 1, 1),
                "sunlight",
                0,
                200
            );
            power.Tick(gameController);
            Assert.Equal(100, battery.Energy);
        }

        [Fact]
        public void TestOverchargeDoesntConsume()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 100, 100);
            ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
            resources.CreateResources("coal", 1);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                resources,
                "coal",
                1,
                200
            );
            power.Tick(gameController);
            Assert.Equal(100, battery.Energy);
            Assert.Equal((uint)1, resources.TotalResources);
        }

        [Fact]
        public void TestChargingTo100Percent()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 0, 100);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                new ResourcesComponentCore(new TestProductionCraftTime(), 1, 1),
                burnResource: "sunlight",
                gainRate: 1
            );
            power.Tick(gameController);
            for (int i = 0; i < 100; i++)
            {
                power.Tick(gameController);
            }
            Assert.Equal(1, battery.PercentEnergy);
            Assert.Equal("100%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestHighGain()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 0, 100);
            PowerComponentCore power = new(
                new WorldObjectCore(null),
                battery,
                new ResourcesComponentCore(new TestProductionCraftTime(), 1, 1),
                gainRate: 95
            );
            power.Tick(gameController);
            Assert.Equal(95, battery.Energy);
            Assert.Equal(Math.Round(0.95, 2), Math.Round(battery.PercentEnergy, 2));
            Assert.Equal("95%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestTwoGeneratorsFourConsumers()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore core1 = this.WorldObject(gameController, 0, 100, gainRate: 10);
            WorldObjectCore core2 = this.WorldObject(gameController, 0, 100, gainRate: 10);
            WorldObjectCore core3 = this.WorldObject(gameController, 0, 100);
            WorldObjectCore core4 = this.WorldObject(gameController, 0, 100);

            core1.power.Tick(gameController);
            core2.power.Tick(gameController);

            // Round 1
            core1.battery.Tick(gameController);
            core2.battery.Tick(gameController);
            core3.battery.Tick(gameController);
            core4.battery.Tick(gameController);

            // Round 2
            core1.battery.Tick(gameController);
            core2.battery.Tick(gameController);
            core3.battery.Tick(gameController);
            core4.battery.Tick(gameController);

            float totalEnergy =
                core1.battery.Energy
                + core2.battery.Energy
                + core3.battery.Energy
                + core4.battery.Energy;

            Assert.Equal(20u, Math.Round(totalEnergy));
            Assert.Equal(5u, Math.Round(core1.battery.Energy));
            Assert.Equal(5u, Math.Round(core2.battery.Energy));
            Assert.Equal(5u, Math.Round(core3.battery.Energy));
            Assert.Equal(5u, Math.Round(core4.battery.Energy));
        }
    }
}
