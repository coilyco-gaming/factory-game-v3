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

        public PowerComponentCore(string burnResource = "", uint burnRate = 0, uint gainRate = 0)
        {
            this.burnResource = burnResource;
            this.burnRate = burnRate;
            this.gainRate = gainRate;
        }

        public List<Dictionary<uint, string>> Tick(
            GameControllerCore gameController,
            WorldObjectCore worldObject
        )
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);
            activity.SetParentId(gameController.backref.WorldObjectTickActivity.Id);

            if (worldObject.battery.PercentEnergy >= 1)
            {
                return new();
            }
            uint resourcesToBurn = worldObject.resources.resources.GetValueOrDefault(
                this.burnResource,
                (uint)0
            );
            if (resourcesToBurn >= this.burnRate)
            {
                // Consume no resources if the battery is full
                try
                {
                    worldObject.battery.Energy += this.gainRate;
                    worldObject.resources.ConsumeResources(this.burnResource, this.burnRate);
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
            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(startingEnergy, capacity);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            if (gainRate != 0)
            {
                worldObject.power = new PowerComponentCore(gainRate: gainRate);
            }
            worldObject.guid = worldObject.CreateGuid();
            gameController.worldObjects ??= new();
            if (!gameController.worldObjects.ContainsKey(worldObject.GridPosition))
            {
                gameController.worldObjects[worldObject.GridPosition] = new();
            }
            gameController.worldObjects[worldObject.GridPosition][worldObject.guid] = worldObject;
            return worldObject;
        }

        [Fact]
        public void TestGeneratePowerZeroes()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };
            WorldObjectCore worldObject = new(null);
            worldObject.battery = new();
            worldObject.resources = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            PowerComponentCore power = new("", 0, 0);
            power.Tick(gameController, worldObject);
            Assert.Equal(0, worldObject.battery.Energy);
            Assert.Equal((uint)0, worldObject.resources.TotalResources);
        }

        [Fact]
        public void TestGeneratePower()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(0, 100);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            worldObject.resources.CreateResources("coal", 1);
            PowerComponentCore power = new("coal", 1, 10);
            power.Tick(gameController, worldObject);
            Assert.Equal(10, worldObject.battery.Energy);
            Assert.Equal((uint)0, worldObject.resources.TotalResources);
        }

        [Fact]
        public void TestConsumeResourcesNoneAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };
            WorldObjectCore worldObject = new(null);
            worldObject.battery = new();
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new("coal", 1, 10);
            power.Tick(gameController, worldObject);
            Assert.Equal(0, worldObject.battery.Energy);
            Assert.Equal((uint)0, worldObject.resources.TotalResources);
        }

        [Fact]
        public void TestConsumeResourcesDifferentAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new();
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new("coal", 1, 10);
            power.Tick(gameController, worldObject);
            Assert.Equal(0, worldObject.battery.Energy);
            Assert.Equal((uint)0, worldObject.resources.TotalResources);
        }

        [Fact]
        public void TestConsumeMoreThanAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new();
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            worldObject.resources.CreateResources("coal", 1);
            PowerComponentCore power = new("coal", 2, 10);
            power.Tick(gameController, worldObject);
            Assert.Equal(0, worldObject.battery.Energy);
            Assert.Equal((uint)1, worldObject.resources.TotalResources);
        }

        [Fact]
        public void TestSolarPower()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(0, 100);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new("sunlight", 0, 10);
            power.Tick(gameController, worldObject);
            Assert.Equal(10, worldObject.battery.Energy);
        }

        [Fact]
        public void TestOvercharge()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(0, 100);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new("sunlight", 0, 200);
            power.Tick(gameController, worldObject);
            Assert.Equal(100, worldObject.battery.Energy);
        }

        [Fact]
        public void TestOverchargeDoesntConsume()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(100, 100);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            worldObject.resources.CreateResources("coal", 1);
            PowerComponentCore power = new("coal", 1, 200);
            power.Tick(gameController, worldObject);
            Assert.Equal(100, worldObject.battery.Energy);
            Assert.Equal((uint)1, worldObject.resources.TotalResources);
        }

        [Fact]
        public void TestChargingTo100Percent()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(0, 100);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new(burnResource: "sunlight", gainRate: 1);
            power.Tick(gameController, worldObject);
            for (int i = 0; i < 100; i++)
            {
                power.Tick(gameController, worldObject);
            }
            Assert.Equal(1, worldObject.battery.PercentEnergy);
            Assert.Equal("100%", worldObject.battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestHighGain()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);
            worldObject.battery = new(0, 100);
            worldObject.resources = new(new TestResourcesGameContent(), 1, 1);
            PowerComponentCore power = new(gainRate: 95);
            power.Tick(gameController, worldObject);
            Assert.Equal(95, worldObject.battery.Energy);
            Assert.Equal(Math.Round(0.95, 2), Math.Round(worldObject.battery.PercentEnergy, 2));
            Assert.Equal("95%", worldObject.battery.PercentEnergyStatus);
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

            core1.power.Tick(gameController, core1);
            core2.power.Tick(gameController, core2);

            // Round 1
            core1.battery.Tick(gameController, core1);
            core2.battery.Tick(gameController, core2);
            core3.battery.Tick(gameController, core3);
            core4.battery.Tick(gameController, core4);

            // Round 2
            core1.battery.Tick(gameController, core1);
            core2.battery.Tick(gameController, core2);
            core3.battery.Tick(gameController, core3);
            core4.battery.Tick(gameController, core4);

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
