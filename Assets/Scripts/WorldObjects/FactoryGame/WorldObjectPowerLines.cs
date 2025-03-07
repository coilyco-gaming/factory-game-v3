using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectPowerLines : WorldObject
    {
        public uint totalBatteryCapacity = 1000;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            base.Instantiate(spawnQueueItem);
            this.core.battery = new(capacity: this.totalBatteryCapacity);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.battery.Balance(this.core, gameController.core);
        }
    }
}
