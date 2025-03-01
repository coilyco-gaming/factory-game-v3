using System;
using Assets.Scripts.Components.Core;
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

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponentCore.StatusData statusData = new()
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
