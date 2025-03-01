using System;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectRadar : WorldObject
    {
        private static uint totalBatteryCapacity = 1000;
        public string Target { get; set; }

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.Battery.Instantiate(capacity: WorldObjectRadar.totalBatteryCapacity);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.Battery.Balance(this, gameController);
        }

        protected override Func<StatusDataComponent.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponent.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = new System.Collections.Generic.Dictionary<string, string>
                    {
                        { "Target", this.Target },
                        { "Energy", this.Battery.PercentEnergyStatus.ToString() },
                    },
                };
                return statusData;
            };
        }
    }
}
