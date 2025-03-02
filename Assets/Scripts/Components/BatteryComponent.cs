namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Linq;

    public class BatteryComponentCore
    {
        private static uint minimumStartingCapacity = 5; // The electrical capacity of empty air... or something. This is really here to prevent infinite charging and NaNs.
        private static double minimumHealth = 0.10f;
        private float energy = 0;

        public float Energy
        {
            get => this.energy;
            set
            {
                this.Degrade();
                this.energy = value > this.Capacity ? this.Capacity : value;
            }
        }

        public uint Capacity { get; set; } = 0;
        public uint StartingCapaity { get; set; } = 0;

        public double PercentEnergy =>
            this.Capacity != 0 //
                ? Math.Round((double)(this.Energy / (double)this.Capacity), 2)
                : 0;

        public string PercentEnergyStatus => $"{this.PercentEnergy * 100}%";

        public double Health =>
            this.StartingCapaity != 0 //
                ? Math.Round((double)(this.Capacity / (double)this.StartingCapaity), 2)
                : 0;

        public bool Healthy => this.Health > BatteryComponentCore.minimumHealth * 2;

        public string HealthStatus => this.Healthy ? "Healthy" : "Unhealthy";

        public void Instantiate(float startingEnergy = 0, uint capacity = 0)
        {
            // Setting a min capacity + rounding to 1 decimal place helps
            // prevent a battery being charged infinitely.
            this.Capacity = capacity == 0 ? (uint)startingEnergy : capacity;
            this.Capacity =
                this.Capacity < BatteryComponentCore.minimumStartingCapacity
                    ? BatteryComponentCore.minimumStartingCapacity
                    : this.Capacity;
            this.StartingCapaity = this.Capacity;
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

            // Instantiate any batteries that haven't been instantiated.
            batteries = batteries
                .Where(battery => battery != null)
                .Select(battery =>
                {
                    if (battery.Capacity == 0)
                    {
                        battery.Instantiate();
                    }
                    return battery;
                })
                .Distinct()
                .ToList();

            // Calculate the total energy and total capacity of all batteries.
            uint totalEnergy = (uint)batteries.Sum(battery => battery.Energy);
            uint totalCapacity = (uint)batteries.Sum(battery => battery.Capacity);

            // Get the target % of battery capacity.
            float targetPercentage = (float)totalEnergy / totalCapacity;

            // Set the energy of each battery to the target % of battery capacity.
            foreach (BatteryComponentCore battery in batteries)
            {
                battery.Energy = battery.Capacity * targetPercentage;
            }
        }

        private void Degrade()
        {
            // Batteries degrade over time, reducing their charging capacity.
            // The degradation happens periodically and occurs when
            // the battery is charged or discharged.
            if (this.Health > BatteryComponentCore.minimumHealth)
            {
                this.Capacity -= 1;
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
    using Assets.Scripts.Unity;
    using Assets.Scripts.WorldObjects.Core;
    using Assets.Scripts.WorldObjects.Unity;
    using UnityEngine;

    public class BatteryComponent : MonoBehaviour
    {
        public readonly BatteryComponentCore core = new();
        public float Energy => this.core.Energy;
        public double PercentEnergy => this.core.PercentEnergy;
        public string PercentEnergyStatus => this.core.PercentEnergyStatus;

        public void Instantiate(uint startingEnergy = 0, uint capacity = 0) =>
            this.core.Instantiate(startingEnergy, capacity);

        // TODO: DRY this pattern, we do it twice
        public void Balance(WorldObject worldObject, GameController gameController)
        {
            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                worldObject.core.GridPosition
            );
            List<BatteryComponentCore> batteries = localWorldObjects
                .Select(localWorldObject => localWorldObject.Battery)
                .ToList();
            this.core.Balance(batteries);
        }
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System;
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
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
        }

        [Fact]
        public void TestBalanceWithNulls()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(startingEnergy: 25, capacity: 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(startingEnergy: 75, capacity: 100);

            battery1.Balance(new List<BatteryComponentCore> { battery2, null, null, null });
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
        }

        [Fact]
        public void TestBalanceOnlyNulls()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(startingEnergy: 25, capacity: 100);

            battery1.Balance(new List<BatteryComponentCore> { null, null, null });
            Assert.Equal(25u, Math.Round(battery1.Energy));
        }

        [Fact]
        public void TestBalanceTwoWithDuplicate()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(25, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(75, 100);

            battery1.Balance(new List<BatteryComponentCore> { battery2, battery1 });
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
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
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
            Assert.Equal(50u, Math.Round(battery3.Energy));
        }

        [Fact]
        public void TestBalanceTwoWithDifferentCapacity()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(25, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(75, 200);

            battery1.Balance(new List<BatteryComponentCore> { battery2 });
            Assert.Equal(33, Math.Round(battery1.Energy));
            Assert.Equal(67, Math.Round(battery2.Energy));
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
            Assert.Equal(33u, Math.Round(battery1.Energy));
            Assert.Equal(66u, Math.Round(battery2.Energy));
            Assert.Equal(99u, Math.Round(battery3.Energy));
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
            Assert.Equal(33u, Math.Round(battery1.Energy));
            Assert.Equal(66u, Math.Round(battery2.Energy));
            Assert.Equal(99u, Math.Round(battery3.Energy));
        }

        [Fact]
        public void TestBalanceTwoEmptyBatteries()
        {
            BatteryComponentCore battery1 = new();
            battery1.Instantiate(0, 100);
            BatteryComponentCore battery2 = new();
            battery2.Instantiate(0, 200);

            battery1.Balance(new List<BatteryComponentCore> { battery2 });
            Assert.Equal(0u, (uint)battery1.Energy);
            Assert.Equal(0u, (uint)battery2.Energy);
        }

        [Fact]
        public void TestBalanceMisconfiguredCapacity()
        {
            BatteryComponentCore battery = new();
            battery.Balance(null);
            Assert.Equal(0u, battery.Energy);
        }

        [Fact]
        public void TestBalanceEmptyList()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(50, 100);
            battery.Balance(new List<BatteryComponentCore>());
            Assert.Equal(50, Math.Round(battery.Energy));
        }

        [Fact]
        public void TestPercentEnergy()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(50, 100);
            Assert.Equal(Math.Round(0.51f, 2), battery.PercentEnergy);
            Assert.Equal("51%", battery.PercentEnergyStatus);
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
            Assert.Equal(4u, battery.Capacity);
            Assert.Equal(0, battery.Energy);
        }

        [Fact]
        public void TestManyChargesDegradeHealth()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            for (int i = 0; i < 10; i++)
            {
                battery.Energy = 10;
            }
            Assert.Equal(89u, battery.Capacity);
            Assert.Equal(Math.Round(0.89, 2), Math.Round(battery.Health, 2));
        }

        [Fact]
        public void TestHealthDegradeHasAFloor()
        {
            BatteryComponentCore battery = new();
            battery.Instantiate(0, 100);
            for (int i = 0; i < 500; i++)
            {
                battery.Energy = 10;
            }
            Assert.Equal(10u, battery.Capacity);
            Assert.Equal(Math.Round(0.1f, 2), Math.Round(battery.Health, 2));
            Assert.False(battery.Healthy);
        }
    }
}
