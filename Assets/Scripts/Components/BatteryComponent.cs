namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;

    public class BatteryComponentCore
    {
        private static uint minimumStartingCapacity = 5; // The electrical capacity of empty air... or something. This is really here to prevent infinite charging and NaNs.
        private static double minimumHealth = 0.10f;
        private float energy = 0;

        public float Energy
        {
            get => this.energy >= 0 ? this.energy : 0;
            set
            {
                this.Degrade();
                bool isNegative = value < 0;
                bool isOverCapacity = value > this.Capacity;
                if (isNegative)
                {
                    this.energy = 0;
                    throw new BatteryCapacityException("Battery empty");
                }
                if (isOverCapacity)
                {
                    this.energy = this.Capacity;
                    throw new BatteryCapacityException("Battery over capacity");
                }
                this.energy = value;
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

        public class BatteryCapacityException : Exception
        {
            public BatteryCapacityException(string message)
                : base(message) { }
        }

        public BatteryComponentCore(float startingEnergy = 0, uint capacity = 0)
        {
            // Setting a min capacity + rounding to 1 decimal place helps
            // prevent a battery being charged infinitely.
            this.Capacity = capacity == 0 ? (uint)startingEnergy : capacity;
            this.Capacity =
                this.Capacity < BatteryComponentCore.minimumStartingCapacity
                    ? BatteryComponentCore.minimumStartingCapacity
                    : this.Capacity;
            this.StartingCapaity = this.Capacity;
            this.energy = startingEnergy;
        }

        // Balance each battery in the list, including yourself,
        // to the same % of battery capacity.
        public void Balance(WorldObjectCore worldObject, GameControllerCore gameController)
        {
            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                worldObject.GridPosition
            );

            List<BatteryComponentCore> batteries = localWorldObjects
                .Select(localWorldObject => localWorldObject.Battery)
                .Distinct()
                .ToList();

            // Add yourself to the list of batteries.
            batteries.Add(this);

            // Instantiate any batteries that haven't been instantiated.
            batteries = batteries.Where(battery => battery != null).Distinct().ToList();

            // Calculate the total energy and total capacity of all batteries.
            uint totalEnergy = (uint)batteries.Sum(battery => battery.Energy);
            uint totalCapacity = (uint)batteries.Sum(battery => battery.Capacity);

            // Get the target % of battery capacity.
            float targetPercentage = (float)totalEnergy / totalCapacity;

            // Set the energy of each battery to the target % of battery capacity.
            foreach (BatteryComponentCore battery in batteries)
            {
                // We set the "inner" battery variable to skip capacity validation.
                battery.Energy = battery.Capacity * targetPercentage;
            }
        }

        private void Degrade()
        {
            // Batteries degrade over time, reducing their charging capacity.
            if (this.Health > BatteryComponentCore.minimumHealth)
            {
                // TODO: swap capacity to a float,
                // TODO: grade by a smaller amount (0.1?)
                this.Capacity -= 1;
            }
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;
    using Xunit;

    public class BatteryComponentTest
    {
        private BatteryComponentCore Battery(
            GameControllerCore gameController,
            uint energy,
            uint capacity
        )
        {
            WorldObjectCore core = new(null)
            {
                Battery = new BatteryComponentCore(energy, capacity),
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            core.Guid = core.CreateGuid();
            gameController.worldObjects ??= new();
            if (!gameController.worldObjects.ContainsKey(core.GridPosition))
            {
                gameController.worldObjects[core.GridPosition] = new();
            }
            gameController.worldObjects[core.GridPosition][core.Guid] = core;
            return core.Battery;
        }

        [Fact]
        public void TestBalanceTwo()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 25, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 75, 100);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
        }

        [Fact]
        public void TestBalanceWithNulls()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 25, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 75, 100);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
        }

        [Fact]
        public void TestBalanceOnlyNulls()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 25, 100);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(25u, Math.Round(battery1.Energy));
        }

        [Fact]
        public void TestBalanceTwoWithDuplicate()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 25, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 75, 100);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
        }

        [Fact]
        public void TestBalanceThree()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 25, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 75, 100);
            BatteryComponentCore battery3 = this.Battery(gameController, 50, 100);

            battery1.Balance(new WorldObjectCore(null), gameController);
            battery2.Balance(new WorldObjectCore(null), gameController);
            battery3.Balance(new WorldObjectCore(null), gameController);

            Assert.Equal(50u, Math.Round(battery1.Energy));
            Assert.Equal(50u, Math.Round(battery2.Energy));
            Assert.Equal(50u, Math.Round(battery3.Energy));
        }

        [Fact]
        public void TestBalanceTwoWithDifferentCapacity()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 25, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 75, 200);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(33, Math.Round(battery1.Energy));
            Assert.Equal(67, Math.Round(battery2.Energy));
        }

        [Fact]
        public void TestBalanceThreeWithDifferentCapacity()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 33, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 66, 200);
            BatteryComponentCore battery3 = this.Battery(gameController, 99, 300);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(33u, Math.Round(battery1.Energy));
            Assert.Equal(66u, Math.Round(battery2.Energy));
            Assert.Equal(99u, Math.Round(battery3.Energy));
        }

        [Fact]
        public void TestBalanceThreeWithDifferentCapacityTwo()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 66, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 66, 200);
            BatteryComponentCore battery3 = this.Battery(gameController, 66, 300);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(33u, Math.Round(battery1.Energy));
            Assert.Equal(66u, Math.Round(battery2.Energy));
            Assert.Equal(99u, Math.Round(battery3.Energy));
        }

        [Fact]
        public void TestBalanceTwoEmptyBatteries()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            BatteryComponentCore battery1 = this.Battery(gameController, 0, 100);
            BatteryComponentCore battery2 = this.Battery(gameController, 0, 200);

            battery1.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(0u, (uint)battery1.Energy);
            Assert.Equal(0u, (uint)battery2.Energy);
        }

        [Fact]
        public void TestBalanceMisconfiguredCapacity()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            BatteryComponentCore battery = new();
            battery.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(0u, battery.Energy);
        }

        [Fact]
        public void TestBalanceEmptyList()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            BatteryComponentCore battery = new(50, 100);
            battery.Balance(new WorldObjectCore(null), gameController);
            Assert.Equal(50, Math.Round(battery.Energy));
        }

        [Fact]
        public void TestPercentEnergy()
        {
            BatteryComponentCore battery = new(50, 100);
            Assert.Equal(Math.Round(0.5, 2), Math.Round(battery.PercentEnergy, 2));
            Assert.Equal("50%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestPercentEnergy9s()
        {
            BatteryComponentCore battery = new(99, 100);
            Assert.Equal(100u, battery.Capacity);
            Assert.Equal(99, battery.Energy);
            Assert.Equal(0.99d, battery.PercentEnergy);
            Assert.Equal("99%", battery.PercentEnergyStatus);
        }

        [Fact]
        public void TestMinCapacity()
        {
            BatteryComponentCore battery = new(0, 0);
            Assert.Equal(5u, battery.Capacity);
            Assert.Equal(0, battery.Energy);
        }

        [Fact]
        public void TestManyChargesDegradeHealth()
        {
            BatteryComponentCore battery = new(0, 100);
            for (int i = 0; i < 10; i++)
            {
                battery.Energy = 10;
            }
            Assert.Equal(90u, battery.Capacity);
            Assert.Equal(Math.Round(0.90, 2), Math.Round(battery.Health, 2));
        }

        [Fact]
        public void TestHealthDegradeHasAFloor()
        {
            BatteryComponentCore battery = new(0, 100);
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
