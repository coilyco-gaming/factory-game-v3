namespace Assets.Scripts.Components.Core
{
    public class ProductionComponentCore
    {
        public void Instantiate() { }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class ProductionComponent : MonoBehaviour
    {
        public readonly ProductionComponentCore core = new();

        public void Instantiate() => this.core.Instantiate();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class ProductionComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            ProductionComponentCore production = new();
            production.Instantiate();
            Assert.True(true);
        }
    }
}
