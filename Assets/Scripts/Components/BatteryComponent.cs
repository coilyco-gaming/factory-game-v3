namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Linq;

    public class BatteryComponentCore
    {
        private static uint minimumCapacity = 100;
        private uint energy = 0;

        public uint Energy
        {
            get => this.energy;
            set => this.energy = value > this.Capacity ? this.Capacity : value;
        }

        public uint Capacity { get; set; } = 0;

        public double PercentEnergy =>
            this.Capacity != 0 ? Math.Round((double)(this.Energy / (double)this.Capacity), 1) : 0;

        public string PercentEnergyStatus => $"{this.PercentEnergy * 100}%";

        public void Instantiate(uint startingEnergy = 0, uint capacity = 0)
        {
            // Setting a min capacity + rounding to 1 decimal place helps
            // prevent a battery being charged infinitely.
            this.Capacity = capacity == 0 ? startingEnergy : capacity;
            this.Capacity =
                this.Capacity < BatteryComponentCore.minimumCapacity
                    ? BatteryComponentCore.minimumCapacity
                    : this.Capacity;
            this.Energy = startingEnergy;
        }

        // TODO: progressively degrade the battey every time its charged, with some cooldown
        // TODO: mark battery as "unhealthy" when current capacity is below 33% of original capacity

        // Balance each battery in the list, including yourself,
        // to the same % of battery capacity.
        public void Balance(List<BatteryComponentCore> batteries)
        {
            batteries ??= new List<BatteryComponentCore>();

            // Add yourself to the list of batteries.
            batteries.Add(this);

            // Filter the list of batteries to remove duplicates
            batteries = batteries.Distinct().ToList();

            // Calculate the total energy and total capacity of all batteries.
            uint totalEnergy = (uint)batteries.Sum(battery => battery.Energy);
            uint totalCapacity = (uint)batteries.Sum(battery => battery.Capacity);

            // Get the target % of battery capacity.
            float targetPercentage = (float)totalEnergy / totalCapacity;

            // Set the energy of each battery to the target % of battery capacity.
            foreach (BatteryComponentCore battery in batteries)
            {
                // This float => int trunctation results in some energy loss.
                // That's perfectly fine, energy transfer shouldn't be perfecty efficient.
                battery.Energy = (uint)(battery.Capacity * targetPercentage);
            }
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.WorldObjects;
    using UnityEngine;

    public class BatteryComponent : MonoBehaviour
    {
        public readonly BatteryComponentCore core = new();
        public uint Energy => this.core.Energy;
        public double PercentEnergy => this.core.PercentEnergy;
        public string PercentEnergyStatus => this.core.PercentEnergyStatus;

        public void Instantiate(uint startingEnergy = 0, uint capacity = 0) =>
            this.core.Instantiate(startingEnergy, capacity);

        // TODO: DRY this pattern, we do it twice
        public void Balance(WorldObject worldObject, GameController gameController)
        {
            List<System.Numerics.Vector2> adjacentTiles = new()
            {
                new System.Numerics.Vector2( // Above
                    worldObject.GridPosition.X + 0,
                    worldObject.GridPosition.Y + 1
                ),
                new System.Numerics.Vector2( // Top Right
                    worldObject.GridPosition.X + 1,
                    worldObject.GridPosition.Y + 1
                ),
                new System.Numerics.Vector2( // Right
                    worldObject.GridPosition.X + 1,
                    worldObject.GridPosition.Y + 0
                ),
                new System.Numerics.Vector2( // Bottom Right
                    worldObject.GridPosition.X + 1,
                    worldObject.GridPosition.Y - 1
                ),
                new System.Numerics.Vector2( // Below
                    worldObject.GridPosition.X + 0,
                    worldObject.GridPosition.Y - 1
                ),
                new System.Numerics.Vector2( // Bottom Left
                    worldObject.GridPosition.X - 1,
                    worldObject.GridPosition.Y - 1
                ),
                new System.Numerics.Vector2( // Left
                    worldObject.GridPosition.X + -1,
                    worldObject.GridPosition.Y + 0
                ),
                new System.Numerics.Vector2( // Top Left
                    worldObject.GridPosition.X + -1,
                    worldObject.GridPosition.Y + 1
                ),
            };
            List<WorldObject> localWorldObjects = adjacentTiles
                .SelectMany(adjacentTile =>
                    gameController.GetWorldObjectsByPosition(adjacentTile)
                    ?? Enumerable.Empty<WorldObject>()
                )
                .ToList();
            List<BatteryComponent> batteries = localWorldObjects
                .Select(localWorldObject => localWorldObject.Battery)
                .ToList();
            this.core.Balance(batteries.Select(battery => battery.core).ToList());
        }
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class BatteryComponentTest
    {
        [Fact]
        public void TestBalanceTwo()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(25, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(75, 100);

            battery1.Balance(new List<BatteryComponentCore> { battery2 });
            Assert.Equal((uint)50, battery1.Energy);
            Assert.Equal((uint)50, battery2.Energy);
        }

        [Fact]
        public void TestBalanceTwoWithDuplicate()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(25, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(75, 100);

            battery1.Balance(new List<BatteryComponentCore> { battery2, battery1 });
            Assert.Equal((uint)50, battery1.Energy);
            Assert.Equal((uint)50, battery2.Energy);
        }

        [Fact]
        public void TestBalanceThree()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(25, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(75, 100);
            BatteryComponentCore battery3 = new();
            battery3.Instantiate(50, 100);

            battery1.Balance(new List<BatteryComponentCore> { battery2, battery3 });
            Assert.Equal((uint)50, battery1.Energy);
            Assert.Equal((uint)50, battery2.Energy);
            Assert.Equal((uint)50, battery3.Energy);
        }

        [Fact]
        public void TestBalanceTwoWithDifferentCapacity()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(25, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(75, 200);

            battery1.Balance(new List<BatteryComponentCore> { battery2 });
            Assert.Equal((uint)33, battery1.Energy);
            Assert.Equal((uint)66, battery2.Energy);
        }

        [Fact]
        public void TestBalanceThreeWithDifferentCapacity()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(33, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(66, 200);
            BatteryComponentCore battery3 = new();
            battery3.Instantiate(99, 300);

            battery1.Balance(new List<BatteryComponentCore> { battery2, battery3 });
            Assert.Equal((uint)33, battery1.Energy);
            Assert.Equal((uint)66, battery2.Energy);
            Assert.Equal((uint)99, battery3.Energy);
        }

        [Fact]
        public void TestBalanceThreeWithDifferentCapacityTwo()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(66, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(66, 200);
            BatteryComponentCore battery3 = new();
            battery3.Instantiate(66, 300);

            battery1.Balance(new List<BatteryComponentCore> { battery2, battery3 });
            Assert.Equal((uint)33, battery1.Energy);
            Assert.Equal((uint)66, battery2.Energy);
            Assert.Equal((uint)99, battery3.Energy);
        }

        [Fact]
        public void TestBalanceTwoEmptyBatteries()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(0, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(0, 200);

            battery1.Balance(new List<BatteryComponentCore> { battery2 });
            Assert.Equal((uint)0, battery1.Energy);
            Assert.Equal((uint)0, battery2.Energy);
        }

        [Fact]
        public void TestBalanceMisconfiguredCapacity()
        {
            BatteryComponentCore battery = new();
            battery.Balance(null);
            Assert.Equal((uint)0, battery.Energy);
        }

        [Fact]
        public void TestBalanceEmptyList()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(50, 100);
            battery.Balance(new List<BatteryComponentCore>());
            Assert.Equal((uint)50, battery.Energy);
        }

        [Fact]
        public void TestPercentEnergy()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(50, 100);
            Assert.Equal(0.5, battery.PercentEnergy);
            Assert.Equal("50%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestPercentEnergy9s()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(99, 100);
            Assert.Equal(1, battery.PercentEnergy);
            Assert.Equal("100%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestMinCapacity()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 0);
            Assert.Equal((uint)100, battery.Capacity);
            Assert.Equal((uint)0, battery.Energy);
        }
    }
}
